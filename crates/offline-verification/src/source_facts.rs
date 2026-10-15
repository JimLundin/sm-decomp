//! Owned, bounded Rust source facts shared by planning and compiler discovery.
//!
//! These are syntax facts, not an authenticated compiler closure or permission to
//! reuse a result. Callers still bind exact source bytes, namespace, compiler
//! dependency output and runtime requirements. Unknown inputs remain explicit.
use anyhow::{Result, ensure};
use proc_macro2::{TokenStream, TokenTree};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};
use syn::{
    parse::Parser,
    visit::{self, Visit},
};
use tool_execution::bounded;

const MAX_SOURCE_BYTES: usize = 8 * 1024 * 1024;
const MAX_TOKENS: usize = 1024 * 1024;
const MAX_NESTING: usize = 96;
const MAX_SYNTAX_SPINE: usize = 512;
const MAX_FACTS: usize = 65536;

/// A syntactically recognized input. Literal arguments are decoded as Rust
/// strings; `None` is dynamic or unsupported syntax, never an absent input.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RustInput {
    FileMacro { name: String, path: Option<String> },
    EnvironmentMacro { name: String, variable: Option<String> },
    ExternalModule { name: String },
    ModulePath { path: Option<String> },
}

/// Immutable ordinary data; parser spans, ASTs and token streams are not retained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustSourceFacts {
    logical_path: PathBuf,
    sha256: String,
    source_bytes: usize,
    identifiers: BTreeSet<String>,
    inputs: BTreeSet<RustInput>,
    file_dependencies: BTreeSet<PathBuf>,
    incomplete: BTreeSet<String>,
}

impl RustSourceFacts {
    pub fn logical_path(&self) -> &Path {
        &self.logical_path
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn source_bytes(&self) -> usize {
        self.source_bytes
    }
    pub fn identifiers(&self) -> &BTreeSet<String> {
        &self.identifiers
    }
    pub fn inputs(&self) -> &BTreeSet<RustInput> {
        &self.inputs
    }
    /// Literal top-level module paths resolved relative to this source. Callers
    /// must read, scan and bind these files; the syntax facts alone prove no
    /// filesystem membership or compiler closure.
    pub fn file_dependencies(&self) -> &BTreeSet<PathBuf> {
        &self.file_dependencies
    }
    pub fn incompleteness(&self) -> &BTreeSet<String> {
        &self.incomplete
    }
    pub fn is_complete(&self) -> bool {
        self.incomplete.is_empty()
    }

    /// Discovery cannot prove an exclusion or narrow execution inputs when the
    /// source contains an unmodeled dependency. This is not compiler acceptance.
    pub fn require_complete(&self) -> Result<()> {
        ensure!(
            self.is_complete(),
            "Rust source has unresolved inputs at {}: {}",
            self.logical_path.display(),
            self.incomplete.iter().cloned().collect::<Vec<_>>().join("; ")
        );
        Ok(())
    }
}

/// Inspect exact bytes within the current bounded deadline. Fixed byte, token,
/// nesting and work ceilings also apply when there is no enclosing IO scope.
/// Syntax errors return incomplete facts; resource/deadline errors return `Err`.
pub fn scan(logical_path: &Path, bytes: &[u8]) -> Result<RustSourceFacts> {
    bounded::limited(|| scan_bounded(logical_path, bytes))
}

fn scan_bounded(logical_path: &Path, bytes: &[u8]) -> Result<RustSourceFacts> {
    bounded::check()?;
    ensure!(bytes.len() <= MAX_SOURCE_BYTES, "Rust source fact byte limit exceeded");
    ensure!(logical_path.as_os_str().len() <= 4096, "Rust source fact path limit exceeded");
    let mut hash = Sha256::new();
    for chunk in bytes.chunks(65536) {
        bounded::check()?;
        hash.update(chunk);
    }
    let mut facts = RustSourceFacts {
        logical_path: logical_path.into(),
        sha256: format!("{:x}", hash.finalize()),
        source_bytes: bytes.len(),
        identifiers: BTreeSet::new(),
        inputs: BTreeSet::new(),
        file_dependencies: BTreeSet::new(),
        incomplete: BTreeSet::new(),
    };
    let Ok(source) = std::str::from_utf8(bytes) else {
        facts.incomplete.insert("Rust source is not UTF-8".into());
        return Ok(facts);
    };
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let tokens = match source.parse::<TokenStream>() {
        Ok(tokens) => tokens,
        Err(error) => {
            bounded::check()?;
            facts.incomplete.insert(format!("Rust tokenization failed: {error}"));
            return Ok(facts);
        }
    };
    // proc-macro2 tokenizes and drops groups iteratively. Bound the token tree
    // before invoking syn's recursive grammar parser or AST visitor.
    check_tokens(&tokens)?;
    let syntax = match syn::parse2::<syn::File>(tokens) {
        Ok(syntax) => syntax,
        Err(error) => {
            bounded::check()?;
            facts.incomplete.insert(format!("Rust syntax parsing failed: {error}"));
            return Ok(facts);
        }
    };
    bounded::check()?;
    let mut scanner = Scanner {
        facts: &mut facts,
        work: 0,
        depth: 0,
        error: None,
        imports: BTreeSet::new(),
        glob_import: false,
        builtins: BTreeSet::new(),
        module_depth: 0,
    };
    scanner.visit_file(&syntax);
    if let Some(error) = scanner.error {
        return Err(error);
    }
    for name in scanner.builtins {
        if scanner.glob_import
            || scanner.imports.contains(&name)
            || scanner.imports.contains("std")
            || scanner.imports.contains("core")
        {
            scanner.facts.incomplete.insert(format!("builtin macro or derive may be shadowed by an import: {name}"));
        }
    }
    ensure!(
        facts.identifiers.len() <= MAX_FACTS && facts.inputs.len() <= MAX_FACTS && facts.incomplete.len() <= MAX_FACTS,
        "Rust source fact inventory limit exceeded"
    );
    bounded::check()?;
    Ok(facts)
}

fn check_tokens(tokens: &TokenStream) -> Result<()> {
    // Each frame tracks tokens since its last syntax separator and the parent
    // grammar spine. Long unary/generic/method chains have shallow delimiters
    // but still need a bound before recursive parsing.
    let mut stack = vec![(tokens.clone().into_iter(), 0usize, 0usize, proc_macro2::Delimiter::None)];
    let mut count = 0usize;
    while let Some((stream, run, parents, delimiter)) = stack.last_mut() {
        let Some(token) = stream.next() else {
            let block = *delimiter == proc_macro2::Delimiter::Brace;
            stack.pop();
            if block {
                if let Some((_, run, _, _)) = stack.last_mut() {
                    *run = 0;
                }
            }
            continue;
        };
        count += 1;
        ensure!(count <= MAX_TOKENS, "Rust source fact token limit exceeded");
        if count % 256 == 0 {
            bounded::check()?;
        }
        if matches!(&token, TokenTree::Punct(p) if matches!(p.as_char(), ';' | ',')) {
            *run = 0;
        } else {
            *run += 1;
        }
        let spine = *parents + *run;
        ensure!(spine <= MAX_SYNTAX_SPINE, "Rust source fact syntax spine limit exceeded");
        if let TokenTree::Group(group) = token {
            ensure!(stack.len() < MAX_NESTING, "Rust source fact nesting limit exceeded");
            stack.push((group.stream().into_iter(), 0, spine, group.delimiter()));
        }
    }
    bounded::check()
}

fn name(ident: &proc_macro2::Ident) -> String {
    let name = ident.to_string();
    name.strip_prefix("r#").unwrap_or(&name).to_owned()
}

fn literal_argument(tokens: TokenStream) -> Option<String> {
    let values = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated.parse2(tokens).ok()?;
    match values.first()? {
        syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(value), .. }) => Some(value.value()),
        _ => None,
    }
}

fn literal_expression(value: &syn::Expr) -> Option<String> {
    match value {
        syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(value), .. }) => Some(value.value()),
        _ => None,
    }
}

const BUILTIN_MACROS: &[&str] = &[
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "matches",
    "vec",
    "panic",
    "unreachable",
    "unimplemented",
    "todo",
    "format",
    "format_args",
    "concat",
    "stringify",
    "file",
    "line",
    "column",
    "module_path",
    "cfg",
    "compile_error",
];
const BUILTIN_DERIVES: &[&str] = &["Clone", "Copy", "Debug", "Default", "Eq", "PartialEq", "Ord", "PartialOrd", "Hash"];
const BUILTIN_ATTRIBUTES: &[&str] = &[
    "allow",
    "warn",
    "deny",
    "forbid",
    "expect",
    "cfg",
    "inline",
    "cold",
    "repr",
    "must_use",
    "deprecated",
    "track_caller",
    "non_exhaustive",
    "target_feature",
    "used",
    "no_mangle",
    "export_name",
    "link_section",
    "test",
    "bench",
    "ignore",
    "should_panic",
    "automatically_derived",
    "unsafe",
];

struct Scanner<'a> {
    facts: &'a mut RustSourceFacts,
    work: usize,
    depth: usize,
    error: Option<anyhow::Error>,
    imports: BTreeSet<String>,
    glob_import: bool,
    builtins: BTreeSet<String>,
    module_depth: usize,
}

impl Scanner<'_> {
    fn step(&mut self) -> bool {
        if self.error.is_some() {
            return false;
        }
        self.work += 1;
        if self.work > MAX_TOKENS * 8 {
            self.error = Some(anyhow::anyhow!("Rust source fact traversal work limit exceeded"));
        } else if self.work % 256 == 0 {
            self.error = bounded::check().err();
        }
        self.error.is_none()
    }

    fn enter(&mut self) -> bool {
        if !self.step() {
            return false;
        }
        if self.depth >= MAX_SYNTAX_SPINE {
            self.error = Some(anyhow::anyhow!("Rust source fact AST nesting limit exceeded"));
            return false;
        }
        self.depth += 1;
        true
    }

    fn unresolved(&mut self, reason: String) {
        self.facts.incomplete.insert(reason);
    }

    fn invocation(&mut self, path: &[String], tokens: TokenStream) {
        let Some(last) = path.last() else {
            return;
        };
        let invocation = path.join("::");
        match last.as_str() {
            "include" | "include_str" | "include_bytes" => {
                let argument = literal_argument(tokens);
                self.facts.inputs.insert(RustInput::FileMacro { name: last.clone(), path: argument.clone() });
                self.unresolved(format!("uncaptured file macro {invocation}!({argument:?})"));
            }
            "env" | "option_env" => {
                let argument = literal_argument(tokens);
                self.facts
                    .inputs
                    .insert(RustInput::EnvironmentMacro { name: last.clone(), variable: argument.clone() });
                self.unresolved(format!("unbound environment macro {invocation}!({argument:?})"));
            }
            value
                if BUILTIN_MACROS.contains(&value)
                    && (path.len() == 1
                        || (path.len() == 2 && ["std", "core", "alloc"].contains(&path[0].as_str()))) =>
            {
                self.builtins.insert(last.clone());
            }
            _ => self.unresolved(format!("unmodeled macro expansion: {invocation}!")),
        }
    }

    fn tokens(&mut self, tokens: TokenStream) {
        let tokens: Vec<_> = tokens.into_iter().collect();
        let mut index = 0usize;
        while index < tokens.len() {
            if !self.step() {
                return;
            }
            // Doc comments become #[doc = "..."] tokens. Literal prose must
            // not manufacture even a dependency on a module named `doc`.
            if matches!(&tokens[index], TokenTree::Punct(p) if p.as_char() == '#') {
                let mut attribute = index + 1;
                if matches!(tokens.get(attribute), Some(TokenTree::Punct(p)) if p.as_char() == '!') {
                    attribute += 1;
                }
                if let Some(TokenTree::Group(group)) = tokens.get(attribute) {
                    if group.delimiter() == proc_macro2::Delimiter::Bracket {
                        if let Ok(syn::Meta::NameValue(value)) = syn::parse2::<syn::Meta>(group.stream()) {
                            if value.path.is_ident("doc") && literal_expression(&value.value).is_some() {
                                index = attribute + 1;
                                continue;
                            }
                        }
                    }
                }
            }
            match &tokens[index] {
                TokenTree::Ident(ident) => {
                    let ident = name(ident);
                    self.facts.identifiers.insert(ident.clone());
                    if ["use", "mod", "extern", "macro_rules", "macro"].contains(&ident.as_str()) {
                        self.unresolved(format!(
                            "unmodeled binding or module syntax in macro/attribute tokens: {ident}"
                        ));
                    }
                    let mut path = vec![ident];
                    let mut start = index;
                    while start >= 3
                        && matches!(&tokens[start - 1], TokenTree::Punct(p) if p.as_char() == ':')
                        && matches!(&tokens[start - 2], TokenTree::Punct(p) if p.as_char() == ':')
                    {
                        let TokenTree::Ident(previous) = &tokens[start - 3] else {
                            break;
                        };
                        path.push(name(previous));
                        start -= 3;
                    }
                    path.reverse();
                    self.path(&path);
                    if matches!(tokens.get(index + 1), Some(TokenTree::Punct(p)) if p.as_char() == '!') {
                        if let Some(TokenTree::Group(group)) = tokens.get(index + 2) {
                            self.invocation(&path, group.stream());
                        }
                    }
                }
                TokenTree::Group(group) => self.tokens(group.stream()),
                _ => {}
            }
            index += 1;
        }
    }

    fn path(&mut self, segments: &[String]) {
        if segments.len() > 1
            && segments
                .iter()
                .any(|part| ["fs", "File", "env", "process", "net", "ffi", "time"].contains(&part.as_str()))
        {
            self.unresolved(format!("unmodeled runtime input or capability: {}", segments.join("::")));
        }
    }

    fn import(&mut self, prefix: &mut Vec<String>, tree: &syn::UseTree) {
        if !self.step() {
            return;
        }
        match tree {
            syn::UseTree::Path(path) => {
                prefix.push(name(&path.ident));
                self.import(prefix, &path.tree);
                prefix.pop();
            }
            syn::UseTree::Name(value) => {
                prefix.push(name(&value.ident));
                self.path(prefix);
                self.imports.insert(name(&value.ident));
                prefix.pop();
            }
            syn::UseTree::Rename(value) => {
                prefix.push(name(&value.ident));
                self.path(prefix);
                self.imports.insert(name(&value.rename));
                prefix.pop();
            }
            syn::UseTree::Glob(_) => {
                self.path(prefix);
                self.glob_import = true;
            }
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.import(prefix, tree);
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for Scanner<'_> {
    fn visit_ident(&mut self, ident: &'ast proc_macro2::Ident) {
        if self.step() {
            self.facts.identifiers.insert(name(ident));
        }
    }
    fn visit_item(&mut self, item: &'ast syn::Item) {
        if !self.enter() {
            return;
        }
        if matches!(item, syn::Item::Verbatim(_)) {
            self.unresolved("unmodeled Rust item syntax".into());
        }
        visit::visit_item(self, item);
        self.depth -= 1;
    }
    fn visit_expr(&mut self, expr: &'ast syn::Expr) {
        if !self.enter() {
            return;
        }
        if matches!(expr, syn::Expr::Verbatim(_)) {
            self.unresolved("unmodeled Rust expression syntax".into());
        }
        visit::visit_expr(self, expr);
        self.depth -= 1;
    }
    fn visit_type(&mut self, ty: &'ast syn::Type) {
        if !self.enter() {
            return;
        }
        if matches!(ty, syn::Type::Verbatim(_)) {
            self.unresolved("unmodeled Rust type syntax".into());
        }
        visit::visit_type(self, ty);
        self.depth -= 1;
    }
    fn visit_pat(&mut self, pat: &'ast syn::Pat) {
        if !self.enter() {
            return;
        }
        if matches!(pat, syn::Pat::Verbatim(_)) {
            self.unresolved("unmodeled Rust pattern syntax".into());
        }
        visit::visit_pat(self, pat);
        self.depth -= 1;
    }
    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        if module.content.is_none() {
            let name = name(&module.ident);
            self.facts.inputs.insert(RustInput::ExternalModule { name: name.clone() });
            // This reviewed form has deterministic source-relative semantics.
            // Implicit modules and inline-module path resolution remain
            // uncertain; cfg_attr is still visited as an unmodeled attribute.
            let paths = module.attrs.iter().filter(|attribute| attribute.path().is_ident("path")).collect::<Vec<_>>();
            if self.module_depth == 0 && paths.len() == 1 {
                let path = match &paths[0].meta {
                    syn::Meta::NameValue(value) => literal_expression(&value.value),
                    _ => None,
                };
                if let Some(path) = path
                    && !path.is_empty()
                    && Path::new(&path).components().all(|part| matches!(part, Component::Normal(_)))
                    && Path::new(&path).extension().is_some_and(|extension| extension == "rs")
                {
                    self.facts
                        .file_dependencies
                        .insert(self.facts.logical_path.parent().unwrap_or(Path::new("")).join(&path));
                    self.facts.inputs.insert(RustInput::ModulePath { path: Some(path) });
                    self.visit_ident(&module.ident);
                    self.visit_visibility(&module.vis);
                    for attribute in &module.attrs {
                        if !attribute.path().is_ident("path") {
                            self.visit_attribute(attribute);
                        }
                    }
                    return;
                }
            }
            self.unresolved(format!("unresolved external Rust module: {name}"));
        }
        self.module_depth += 1;
        visit::visit_item_mod(self, module);
        self.module_depth -= 1;
    }
    fn visit_item_foreign_mod(&mut self, module: &'ast syn::ItemForeignMod) {
        self.unresolved("unmodeled foreign linkage/runtime inputs".into());
        visit::visit_item_foreign_mod(self, module);
    }
    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        if !["std", "core", "alloc"].contains(&name(&item.ident).as_str()) {
            self.unresolved(format!("unmodeled external crate: {}", item.ident));
        }
        if let Some((_, alias)) = &item.rename {
            self.imports.insert(name(alias));
        }
        visit::visit_item_extern_crate(self, item);
    }
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        if let syn::UseTree::Path(path) = &item.tree {
            let origin = name(&path.ident);
            if !["crate", "self", "super", "std", "core", "alloc"].contains(&origin.as_str()) {
                self.unresolved(format!("unresolved import origin: {origin}"));
            }
        }
        self.import(&mut vec![], &item.tree);
        visit::visit_item_use(self, item);
    }
    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.path(&path.segments.iter().map(|segment| name(&segment.ident)).collect::<Vec<_>>());
        visit::visit_path(self, path);
    }
    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        if !self.step() {
            return;
        }
        let path = invocation.path.segments.iter().map(|segment| name(&segment.ident)).collect::<Vec<_>>();
        self.invocation(&path, invocation.tokens.clone());
        self.visit_path(&invocation.path);
        self.tokens(invocation.tokens.clone());
    }
    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if !self.step() {
            return;
        }
        let path = attribute.path().segments.iter().map(|segment| name(&segment.ident)).collect::<Vec<_>>();
        let attribute_name = path.join("::");
        if attribute.path().is_ident("doc") {
            if let syn::Meta::NameValue(value) = &attribute.meta {
                if literal_expression(&value.value).is_some() {
                    return;
                }
            }
            self.unresolved("macro-valued or unsupported doc attribute".into());
        } else if attribute.path().is_ident("path") {
            let path = match &attribute.meta {
                syn::Meta::NameValue(value) => literal_expression(&value.value),
                _ => None,
            };
            self.facts.inputs.insert(RustInput::ModulePath { path: path.clone() });
            self.unresolved(format!("unresolved module path attribute: {path:?}"));
        } else if attribute.path().is_ident("derive") {
            let derives =
                attribute.parse_args_with(syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated);
            match derives {
                Ok(derives) => {
                    for derive in derives {
                        let name =
                            derive.segments.iter().map(|segment| name(&segment.ident)).collect::<Vec<_>>().join("::");
                        if BUILTIN_DERIVES.contains(&name.as_str()) {
                            self.builtins.insert(name);
                        } else {
                            self.unresolved(format!("unmodeled derive expansion: {name}"));
                        }
                    }
                }
                Err(_) => self.unresolved("unparsed derive expansion".into()),
            }
        } else if !BUILTIN_ATTRIBUTES.contains(&attribute_name.as_str()) {
            self.unresolved(format!("unmodeled attribute expansion: {attribute_name}"));
        }
        match &attribute.meta {
            syn::Meta::List(list) => self.tokens(list.tokens.clone()),
            syn::Meta::NameValue(value) => self.visit_expr(&value.value),
            syn::Meta::Path(_) => {}
        }
    }
}
