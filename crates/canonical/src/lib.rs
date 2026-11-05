//! `#[derive(Canonical)]`: the one Checkpoint codec and diff for every State type (K6).
//!
//! The trait and its primitive impls live in `game::canonical`, beside the State it encodes.
//! This crate only writes the per-type impls, so no State type has a hand-written codec.
//! Generated code names `::game::canonical`, so `game` declares `extern crate self as game;`.
//!
//! The encoding, per shape:
//! - a struct or newtype encodes its fields in declaration order, with no framing;
//! - an enum encodes a one-byte tag, the variant's position in declaration order (not its
//!   discriminant), then that variant's fields;
//! - a field-less struct encodes nothing.
//!
//! `diff` names the first differing field by path. A lone unnamed field (a newtype, or a
//! variant like `Rising(Speed)`) is transparent: the path stops at the newtype's own name.
//!
//! The macro reads its input with `proc_macro` alone. It only needs field and variant names,
//! never types, so it takes no `syn` or `quote`, whose build scripts can't run on locked-down
//! machines. A generic type's impl bounds every type parameter by `Canonical`, as the standard
//! derives do; const parameters pass through and lifetimes are refused.

use proc_macro::{Delimiter, Group, Spacing, TokenStream, TokenTree};

/// The path every generated impl uses for the trait and its helpers.
const CODEC: &str = "::game::canonical";

/// Derives `game::canonical::Canonical` for a struct, a newtype or an enum with data.
#[proc_macro_derive(Canonical)]
pub fn derive_canonical(input: TokenStream) -> TokenStream {
    let code = match Item::parse(input) {
        Ok((item, generics)) => item.expand(&generics),
        Err(message) => format!("::core::compile_error!({message:?});"),
    };
    code.parse().unwrap_or_default()
}

/// The fields of a struct or of one variant.
enum Shape {
    Named(Vec<String>),
    Unnamed(usize),
    Unit,
}

/// A generic item's parameters, as the impl header needs them.
#[derive(Default)]
struct Generics {
    /// The parameters as declared, for `impl<..>`.
    declared: String,
    /// The parameter names, for `Name<..>`.
    arguments: String,
    /// One `T: Canonical` per type parameter, for the where clause.
    bounds: String,
}

impl Generics {
    /// Reads the parameters between a type's `<` and `>`.
    fn parse(tokens: &[TokenTree]) -> Result<Self, String> {
        let mut arguments = Vec::new();
        let mut bounds = Vec::new();
        for parameter in split_commas(tokens.iter().cloned().collect()) {
            match parameter.as_slice() {
                [TokenTree::Ident(keyword), TokenTree::Ident(name), ..]
                    if keyword.to_string() == "const" =>
                {
                    arguments.push(name.to_string());
                }
                [TokenTree::Ident(name), ..] => {
                    bounds.push(format!("{name}: {CODEC}::Canonical"));
                    arguments.push(name.to_string());
                }
                _ => return Err("Canonical takes type and const parameters only".to_owned()),
            }
        }
        Ok(Self {
            declared: tokens.iter().cloned().collect::<TokenStream>().to_string(),
            arguments: arguments.join(", "),
            bounds: bounds.join(", "),
        })
    }
}

/// Splits `<..> rest` into the parameters inside the angle brackets and the rest.
fn split_generics(tokens: &[TokenTree]) -> Option<(&[TokenTree], &[TokenTree])> {
    let mut depth = 0_usize;
    let mut after_minus = false;
    for (index, token) in tokens.iter().enumerate() {
        if let TokenTree::Punct(punct) = token {
            match punct.as_char() {
                '<' => depth = depth.saturating_add(1),
                '>' if !after_minus => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        let rest = tokens.get(index.checked_add(1)?..)?;
                        return Some((tokens.get(1..index)?, rest));
                    }
                }
                _ => {}
            }
            after_minus = punct.as_char() == '-' && punct.spacing() == Spacing::Joint;
        } else {
            after_minus = false;
        }
    }
    None
}

/// The item the derive is attached to: only what the codec needs.
enum Item {
    Struct {
        name: String,
        shape: Shape,
    },
    Enum {
        name: String,
        variants: Vec<(String, Shape)>,
    },
}

/// One field: how generated code reaches it, and the path segment its diff adds.
struct Field {
    /// The member in `self.member` and in patterns: a name or a position.
    member: String,
    /// `None` for a lone unnamed field, which is transparent in paths.
    segment: Option<String>,
}

impl Shape {
    fn fields(&self) -> Vec<Field> {
        match self {
            Self::Named(names) => names
                .iter()
                .map(|name| Field {
                    member: name.clone(),
                    segment: Some(name.trim_start_matches("r#").to_owned()),
                })
                .collect(),
            Self::Unnamed(count) => (0..*count)
                .map(|position| Field {
                    member: position.to_string(),
                    segment: (*count > 1).then(|| position.to_string()),
                })
                .collect(),
            Self::Unit => Vec::new(),
        }
    }

    /// The pattern `path { member: prefix0, .. }`, `path(prefix0, ..)` or `path`.
    fn pattern(&self, path: &str, prefix: &str) -> String {
        let fields = self.fields();
        let bindings = fields
            .iter()
            .enumerate()
            .map(|(position, field)| (field.member.clone(), format!("{prefix}{position}")));
        match self {
            Self::Named(_) => {
                let members: Vec<String> = bindings
                    .map(|(member, binding)| format!("{member}: {binding}"))
                    .collect();
                format!("{path} {{ {} }}", members.join(", "))
            }
            Self::Unnamed(_) => {
                let members: Vec<String> = bindings.map(|(_, binding)| binding).collect();
                format!("{path}({})", members.join(", "))
            }
            Self::Unit => path.to_owned(),
        }
    }

    /// The expression that decodes this shape's fields in order into `path`.
    fn decoded(&self, path: &str) -> String {
        let decode = format!("{CODEC}::Canonical::decode(input)?");
        match self {
            Self::Named(names) => {
                let members: Vec<String> = names
                    .iter()
                    .map(|name| format!("{name}: {decode}"))
                    .collect();
                format!("{path} {{ {} }}", members.join(", "))
            }
            Self::Unnamed(count) => format!("{path}({})", vec![decode; *count].join(", ")),
            Self::Unit => path.to_owned(),
        }
    }
}

/// The diff step for one pair of fields: return the first difference, extended by its path.
fn diff_step(left: &str, right: &str, segment: Option<&str>, variant: Option<&str>) -> String {
    let field = segment
        .map(|label| format!(".within({CODEC}::Segment::Field({label:?}))"))
        .unwrap_or_default();
    let variant = variant
        .map(|label| format!(".within({CODEC}::Segment::Variant({label:?}))"))
        .unwrap_or_default();
    format!(
        "if let ::core::option::Option::Some(difference) = {CODEC}::Canonical::diff({left}, {right}) {{ \
         return ::core::option::Option::Some(difference{field}{variant}); }}"
    )
}

impl Item {
    fn parse(input: TokenStream) -> Result<(Self, Generics), String> {
        let tokens: Vec<TokenTree> = input.into_iter().collect();
        let [TokenTree::Ident(keyword), TokenTree::Ident(name), rest @ ..] =
            skip_attributes(&tokens)
        else {
            return Err("Canonical expects a struct or an enum".to_owned());
        };
        let name = name.to_string();
        let (generics, rest) = match rest {
            [TokenTree::Punct(open), ..] if open.as_char() == '<' => {
                let (parameters, rest) = split_generics(rest)
                    .ok_or_else(|| format!("Canonical can't read the generics of {name}"))?;
                (Generics::parse(parameters)?, rest)
            }
            _ => (Generics::default(), rest),
        };
        Self::parse_body(&keyword.to_string(), name, rest).map(|item| (item, generics))
    }

    fn parse_body(keyword: &str, name: String, rest: &[TokenTree]) -> Result<Self, String> {
        match (keyword, rest) {
            ("struct", [TokenTree::Group(body)]) if body.delimiter() == Delimiter::Brace => {
                Ok(Self::Struct {
                    name,
                    shape: Shape::Named(named_fields(body)?),
                })
            }
            ("struct", [TokenTree::Group(body), TokenTree::Punct(end)])
                if body.delimiter() == Delimiter::Parenthesis && end.as_char() == ';' =>
            {
                Ok(Self::Struct {
                    name,
                    shape: Shape::Unnamed(split_commas(body.stream()).len()),
                })
            }
            ("struct", [TokenTree::Punct(end)]) if end.as_char() == ';' => Ok(Self::Struct {
                name,
                shape: Shape::Unit,
            }),
            ("enum", [TokenTree::Group(body)]) if body.delimiter() == Delimiter::Brace => {
                let variants = split_commas(body.stream())
                    .iter()
                    .map(|tokens| variant(tokens))
                    .collect::<Result<Vec<_>, _>>()?;
                if variants.is_empty() {
                    return Err(format!("Canonical needs at least one variant: {name}"));
                }
                Ok(Self::Enum { name, variants })
            }
            _ => Err(format!(
                "Canonical expects a struct or an enum with no where clause: {name}"
            )),
        }
    }

    fn expand(&self, generics: &Generics) -> String {
        let Generics {
            declared,
            arguments,
            bounds,
        } = generics;
        let (name, (encode, decode, diff)) = match self {
            Self::Struct { name, shape } => (name, expand_struct(shape)),
            Self::Enum { name, variants } => match expand_enum(name, variants) {
                Ok(parts) => (name, parts),
                Err(message) => return format!("::core::compile_error!({message:?});"),
            },
        };
        format!(
            "#[automatically_derived] impl<{declared}> {CODEC}::Canonical for {name}<{arguments}> where {bounds} {{ \
             fn encode(&self, out: &mut ::std::vec::Vec<u8>) {{ {encode} }} \
             fn decode(input: &mut {CODEC}::Reader<'_>) -> ::core::result::Result<Self, {CODEC}::DecodeError> {{ {decode} }} \
             fn diff(&self, other: &Self) -> ::core::option::Option<{CODEC}::Difference> {{ {diff} }} }}"
        )
    }
}

fn expand_struct(shape: &Shape) -> (String, String, String) {
    let fields = shape.fields();
    if fields.is_empty() {
        return (
            "let _ = out;".to_owned(),
            "let _ = input; ::core::result::Result::Ok(Self {})".to_owned(),
            "let _ = (self, other); ::core::option::Option::None".to_owned(),
        );
    }
    let encode = fields
        .iter()
        .map(|field| format!("{CODEC}::Canonical::encode(&self.{}, out);", field.member))
        .collect::<Vec<_>>()
        .concat();
    let members: Vec<String> = fields
        .iter()
        .map(|field| format!("{}: {CODEC}::Canonical::decode(input)?", field.member))
        .collect();
    let decode = format!(
        "::core::result::Result::Ok(Self {{ {} }})",
        members.join(", ")
    );
    let steps: String = fields
        .iter()
        .map(|field| {
            diff_step(
                &format!("&self.{}", field.member),
                &format!("&other.{}", field.member),
                field.segment.as_deref(),
                None,
            )
        })
        .collect();
    (
        encode,
        decode,
        format!("{steps} ::core::option::Option::None"),
    )
}

fn expand_enum(
    name: &str,
    variants: &[(String, Shape)],
) -> Result<(String, String, String), String> {
    let single = variants.len() == 1;
    let (mut encode, mut decode, mut diff) = (Vec::new(), Vec::new(), Vec::new());
    for (position, (variant, shape)) in variants.iter().enumerate() {
        let tag = u8::try_from(position)
            .map_err(|_| format!("Canonical encodes at most 256 variants: {name}"))?;
        let path = format!("Self::{variant}");
        let label = variant.trim_start_matches("r#");
        let (left, right) = (
            shape.pattern(&path, "left_"),
            shape.pattern(&path, "right_"),
        );
        let fields = shape.fields();
        let encode_fields = (0..fields.len())
            .map(|position| format!("{CODEC}::Canonical::encode(left_{position}, out);"))
            .collect::<Vec<_>>()
            .concat();
        encode.push(format!(
            "{left} => {{ out.push({tag}u8); {encode_fields} }}"
        ));
        decode.push(format!(
            "{tag}u8 => ::core::result::Result::Ok({}),",
            shape.decoded(&path)
        ));
        let bind_other = if single {
            format!("let {right} = other;")
        } else {
            format!(
                "let {right} = other else {{ return ::core::option::Option::Some({CODEC}::Difference::here(self, other)); }};"
            )
        };
        let steps: String = fields
            .iter()
            .enumerate()
            .map(|(position, field)| {
                diff_step(
                    &format!("left_{position}"),
                    &format!("right_{position}"),
                    field.segment.as_deref(),
                    Some(label),
                )
            })
            .collect();
        diff.push(format!(
            "{left} => {{ {bind_other} {steps} ::core::option::Option::None }}"
        ));
    }
    Ok((
        format!("match self {{ {} }}", encode.concat()),
        format!(
            "match <u8 as {CODEC}::Canonical>::decode(input)? {{ {} \
             tag => ::core::result::Result::Err({CODEC}::DecodeError::UnknownVariant {{ type_name: {name:?}, tag }}), }}",
            decode.concat()
        ),
        format!("match self {{ {} }}", diff.concat()),
    ))
}

/// The tokens after any leading attributes and visibility.
fn skip_attributes(mut tokens: &[TokenTree]) -> &[TokenTree] {
    loop {
        tokens = match tokens {
            [
                TokenTree::Punct(hash),
                TokenTree::Group(attribute),
                rest @ ..,
            ] if hash.as_char() == '#' && attribute.delimiter() == Delimiter::Bracket => rest,
            [TokenTree::Ident(public), TokenTree::Group(scope), rest @ ..]
                if public.to_string() == "pub" && scope.delimiter() == Delimiter::Parenthesis =>
            {
                rest
            }
            [TokenTree::Ident(public), rest @ ..] if public.to_string() == "pub" => rest,
            _ => return tokens,
        };
    }
}

/// The names of a braced field list.
fn named_fields(body: &Group) -> Result<Vec<String>, String> {
    split_commas(body.stream())
        .iter()
        .map(|tokens| match skip_attributes(tokens) {
            [TokenTree::Ident(name), TokenTree::Punct(colon), ..] if colon.as_char() == ':' => {
                Ok(name.to_string())
            }
            _ => Err("Canonical expects `name: Type` fields".to_owned()),
        })
        .collect()
}

/// One enum variant's name and fields. A `= discriminant` is ignored: the tag is the position.
fn variant(tokens: &[TokenTree]) -> Result<(String, Shape), String> {
    match skip_attributes(tokens) {
        [TokenTree::Ident(name)] => Ok((name.to_string(), Shape::Unit)),
        [TokenTree::Ident(name), TokenTree::Punct(equals), ..] if equals.as_char() == '=' => {
            Ok((name.to_string(), Shape::Unit))
        }
        [TokenTree::Ident(name), TokenTree::Group(body), ..]
            if body.delimiter() == Delimiter::Brace =>
        {
            Ok((name.to_string(), Shape::Named(named_fields(body)?)))
        }
        [TokenTree::Ident(name), TokenTree::Group(body), ..]
            if body.delimiter() == Delimiter::Parenthesis =>
        {
            Ok((
                name.to_string(),
                Shape::Unnamed(split_commas(body.stream()).len()),
            ))
        }
        _ => Err("Canonical can't read an enum variant".to_owned()),
    }
}

/// Splits a list at its top-level commas, dropping empty entries (a trailing comma).
///
/// Commas inside `<..>` belong to a type such as `Map<K, V>`, so angle brackets are counted;
/// the `>` of `->` doesn't close one.
fn split_commas(stream: TokenStream) -> Vec<Vec<TokenTree>> {
    let mut entries = vec![Vec::new()];
    let mut depth = 0_usize;
    let mut after_minus = false;
    for token in stream {
        if let TokenTree::Punct(punct) = &token {
            match punct.as_char() {
                ',' if depth == 0 => {
                    entries.push(Vec::new());
                    after_minus = false;
                    continue;
                }
                '<' => depth = depth.saturating_add(1),
                '>' if !after_minus => depth = depth.saturating_sub(1),
                _ => {}
            }
            after_minus = punct.as_char() == '-' && punct.spacing() == Spacing::Joint;
        } else {
            after_minus = false;
        }
        if let Some(entry) = entries.last_mut() {
            entry.push(token);
        }
    }
    entries.retain(|entry| !entry.is_empty());
    entries
}
