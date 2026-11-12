# lint_scan.awk: a library of code_check.sh's style rules over one source at a time, loaded with
# -f by source_scan.awk. lint_reset() starts a file, lint_line(LINE, LINENO) takes each of its
# lines already stripped by strip_line (comments gone, strings blanked to "", char literals to
# ''), and lint_file(TEST) reports, calling the host's finding(TAG, DETAIL) once per finding;
# TEST is 1 when the whole file is test code (a tests/ directory or a tests.rs file). A token
# lexer and the size counter's regions, not a parser.
#   -v denied="..."  lints the workspace denies by name (and its denied groups), space-separated
#
# lint-attr      (L1) an #[allow] (the workspace denies clippy::allow_attributes), an
#                #[expect]/#[allow]/#[warn] without reason = "...", or, outside test code, one
#                that names a lint in `denied`. A lint denied only through a group (pedantic,
#                nursery) may be expected, with its reason.
# magic-literal  (L2) a number other than 0 or 1 in logic. Exempt: test code, attributes,
#                struct/enum/union/type items, a constant's head, a constant's value while it is
#                data (the counter's L3: literals, paths, tuples, arrays, struct literals and
#                calls whose arguments are data), the value of an exhaustive lookup's arm (the
#                counter's lookups: a guard-free `match` of two or more arms, or an `enum_map!`,
#                each arm a variant, constant or literal key, keys nested in a tuple variant
#                with no binding included, `=>` and a data value naming no
#                local; a key's own number is still refused), an array length (`[T; N]`,
#                `[x; N]`) and a tuple field (`.2`).
# variant-import (L4) a `use` outside test code that imports an enum's variants: `use Kind::A`,
#                `use Kind::{A, B}` or `use Kind::*`, at any depth of a use tree. A capitalised
#                segment followed by `::` names a type, and a variant is all a `use` can take
#                from one. A group holding only `self` imports the enum itself and passes.
function isid(ch) { return ch ~ /^[A-Za-z0-9_]$/ }
function add(k, t) { n++; kind[n] = k; text[n] = t; lineof[n] = lint_lineno }
# Every token array is emptied: the analysis reads a token or two past the last, which must not
# be the previous file's.
function lint_reset() {
  n = 0; instr = 0; lint_lineno = 0
  split("", kind); split("", text); split("", lineof); split("", region)
}
BEGIN {
  nops = split("<<= >>= ... ..= :: -> => == != <= >= && || += -= *= /= %= ^= &= |= << >> ..", ops, " ")
  nd = split(denied, dl, " ")
  for (i = 1; i <= nd; i++) refused[dl[i]] = 1
  n = 0
}
function lint_line(input, lineno) {
  lint_lineno = lineno
  line = input; len = length(line); i = 1
  # A string that spans lines is one literal: its first line added it; its closing `"` ends it.
  if (instr) { j = index(line, "\""); if (!j) return; instr = 0; i = j + 1 }
  while (i <= len) {
    c = substr(line, i, 1)
    if (c ~ /[ \t\r]/) { i++; continue }
    if (c == "\"") {
      j = index(substr(line, i + 1), "\""); add("lit", "\"\"")
      if (j) i += j + 1; else { instr = 1; i = len + 1 }
      continue
    }
    if (c == "'") {
      if (substr(line, i + 1, 1) == "'") { add("lit", "''"); i += 2; continue }
      j = i + 1; while (j <= len && isid(substr(line, j, 1))) j++
      add("life", substr(line, i, j - i)); i = j; continue
    }
    if (c ~ /[0-9]/) {
      field = (n > 0 && kind[n] == "punct" && text[n] == ".")
      j = i; prev = ""
      while (j <= len) {
        d = substr(line, j, 1)
        if (d ~ /[A-Za-z0-9_]/ || (d ~ /[+-]/ && prev ~ /[eE]/ && substr(line, i, 2) != "0x")) { prev = d; j++; continue }
        if (d == "." && !field && substr(line, j + 1, 1) ~ /[0-9]/) { prev = d; j++; continue }
        break
      }
      add(field ? "field" : "num", substr(line, i, j - i)); i = j; continue
    }
    if (isid(c)) {
      j = i; while (j <= len && isid(substr(line, j, 1))) j++
      add("word", substr(line, i, j - i)); i = j; continue
    }
    if (c ~ /[([{]/) { add("open", c); i++; continue }
    if (c ~ /[])}]/) { add("close", c); i++; continue }
    op = c
    for (k = 1; k <= nops; k++) if (substr(line, i, length(ops[k])) == ops[k]) { op = ops[k]; break }
    add("punct", op); i += length(op)
  }
}
# The index after the bracket group opened at a.
function after_group(a,   d) {
  d = 0
  for (; a <= n; a++) {
    if (kind[a] == "open") d++
    else if (kind[a] == "close") { d--; if (d <= 0) return a + 1 }
  }
  return a
}
# The index after the item starting at a: past its depth-0 `;` or the brace closing its body.
function after_item(a,   d, top) {
  d = 0; top = ""
  for (; a <= n; a++) {
    if (kind[a] == "open") { if (d == 0) top = text[a]; d++ }
    else if (kind[a] == "close") { d--; if (d == 0 && top == "{") return a + 1 }
    else if (kind[a] == "punct" && text[a] == ";" && d == 0) return a + 1
  }
  return a
}
function ends_item(p) {
  return p == 0 || (kind[p] == "punct" && text[p] == ";") || (kind[p] == "open" && text[p] == "{") \
    || kind[p] == "close" || (kind[p] == "word" && (text[p] == "pub" || text[p] == "mut"))
}
# Whether the tokens a..b-1, a constant's value, hold behaviour, as the size counter's L3 decides:
# anything but one data expression. Data is a literal, a minus on a literal, a reference to data,
# a path (a constant, an enum variant, a unit struct), a tuple, an array (`[a, b]`, `[a; N]`), a
# struct literal whose fields are data, a call of a path whose every argument is data
# (`Picture::new(0x009)`), or a range whose two ends are literals or paths (`0x280..0x500`). A
# method call on a value, a field, an index, a turbofish, control flow, a closure, a macro and
# every other operator are logic.
function behaviour(a, b) { at = a; lim = b; return !(data_expr() && at == lim) }
function is_kw(w) {
  return w ~ /^(as|async|await|break|const|continue|dyn|else|enum|extern|fn|for|if|impl|in|let|loop|match|mod|move|mut|pub|ref|return|static|struct|trait|type|union|unsafe|use|where|while|yield)$/
}
function tok(k, t) { return at < lim && kind[at] == k && (t == "" || text[at] == t) }
# Each data_* advances `at` past one form and returns 1, or returns 0.
function data_expr(   from) {
  from = at
  if (data_range_end() && (tok("punct", "..") || tok("punct", "..="))) {
    at++
    if (data_range_end()) return 1
    return 0
  }
  at = from
  if (tok("num") || tok("lit")) { at++; return 1 }
  if (tok("punct", "-")) { at++; if (tok("num") || tok("lit")) { at++; return 1 } return 0 }
  if (tok("punct", "&")) { at++; return data_expr() }
  if (tok("open", "(")) { at++; return data_list() }
  if (tok("open", "[")) { at++; return data_array() }
  if (tok("word") || tok("punct", "::")) {
    if (!data_path()) return 0
    if (tok("open", "(")) { at++; return data_list() }
    if (tok("open", "{")) { at++; return data_fields() }
    return 1
  }
  return 0
}
function data_path() {
  if (tok("punct", "::")) at++
  while (1) {
    if (!tok("word") || is_kw(text[at])) return 0
    at++
    if (!tok("punct", "::")) return 1
    at++
  }
}
# A range's end: a literal (a minus on one included) or a path that neither calls nor builds.
function data_range_end() {
  if (tok("num") || tok("lit")) { at++; return 1 }
  if (tok("punct", "-")) { at++; if (tok("num") || tok("lit")) { at++; return 1 } return 0 }
  if (!tok("word") && !tok("punct", "::")) return 0
  if (!data_path()) return 0
  return !tok("open", "(") && !tok("open", "{")
}
function data_list() {
  while (1) {
    if (tok("close")) { at++; return 1 }
    if (!data_expr()) return 0
    if (tok("punct", ",")) { at++; continue }
    if (tok("close")) { at++; return 1 }
    return 0
  }
}
function data_array(   from) {
  from = at
  if (data_expr() && tok("punct", ";")) {
    at++
    if (!data_expr() || !tok("close")) return 0
    at++; return 1
  }
  at = from
  return data_list()
}
function data_fields() {
  while (1) {
    if (tok("close")) { at++; return 1 }
    if (tok("punct", "..")) {
      at++
      if (!data_expr() || !tok("close")) return 0
      at++; return 1
    }
    if (!tok("word") && !tok("num")) return 0
    at++
    if (tok("punct", ":")) { at++; if (!data_expr()) return 0 }
    if (tok("punct", ",")) { at++; continue }
    if (tok("close")) { at++; return 1 }
    return 0
  }
}
function mark(a, b, r,   k) { for (k = a; k < b; k++) region[k] = r }
# The first token of a..b-1 outside every bracket group whose text is t, or 0.
function top_find(a, b, t,   d) {
  d = 0
  for (; a < b; a++) {
    if (d == 0 && text[a] == t && kind[a] != "lit") return a
    if (kind[a] == "open") d++
    else if (kind[a] == "close") d--
  }
  return 0
}
# A literal key, a minus on one included; advances `at` as data_* do.
function key_lit() {
  if (tok("num") || tok("lit")) { at++; return 1 }
  if (tok("punct", "-")) { at++; if (tok("num") || tok("lit")) { at++; return 1 } }
  return 0
}
# Whether a..b-1 is one key of a lookup's pattern: a literal, a range of two literals, or a path
# to a variant or a constant (its last part capitalised), which may hold, as a tuple variant's
# fields, patterns that are keys in turn (`Outer::A(Inner::B, _)`). A key binds nothing.
# lookup_pattern recurses through here, so nothing below the recursion reads `at` or `lim`.
function lookup_key(a, b,   p, e, k, d, s) {
  at = a; lim = b
  if (key_lit()) {
    if (at == lim) return 1
    if (!tok("punct", "..") && !tok("punct", "..=")) return 0
    at++
    return key_lit() && at == lim
  }
  at = a
  if (!data_path() || text[at - 1] !~ /^[A-Z]/) return 0
  p = at
  if (p == b) return 1
  if (kind[p] != "open" || text[p] != "(" || after_group(p) != b) return 0
  d = 0; s = p + 1
  for (k = p + 1; k < b; k++) {
    if (k < b - 1 && kind[k] == "open") { d++; continue }
    if (k < b - 1 && kind[k] == "close") { d--; continue }
    if (k < b - 1 && !(d == 0 && kind[k] == "punct" && text[k] == ",")) continue
    if (k > s && !lookup_pattern(s, k)) return 0
    s = k + 1
  }
  return 1
}
# Whether a..b-1 is a lookup's pattern: `_`, or keys joined by `|`.
function lookup_pattern(a, b,   k, d, s) {
  if (b == a + 1 && text[a] == "_") return 1
  d = 0; s = a
  for (k = a; k < b; k++) {
    if (kind[k] == "open") d++
    else if (kind[k] == "close") d--
    else if (d == 0 && kind[k] == "punct" && text[k] == "|") { if (!lookup_key(s, k)) return 0; s = k + 1 }
  }
  return lookup_key(s, b)
}
# Whether a..b-1 names a local: a lowercase word that is no part of a path, no function it calls,
# no field's label, and not true or false.
function names_local(a, b,   k) {
  for (k = a; k < b; k++) {
    if (kind[k] != "word" || text[k] !~ /^[a-z_]/ || text[k] ~ /^(true|false)$/) continue
    if (k > a && text[k - 1] == "::") continue
    if (k + 1 < b && (text[k + 1] == "(" || text[k + 1] == "::" || text[k + 1] == ":")) continue
    return 1
  }
  return 0
}
# Whether a..b-1 is a lookup's value: data by the constant rule that names no local.
function lookup_value(a, b) { return b > a && !behaviour(a, b) && !names_local(a, b) }
# The rows of a lookup in a..b-1, separated by commas, each a pattern, `=>` and a value; -1 if
# one is not. Their values' spans are left in vs[1..nv], ve[1..nv].
function lookup_rows(a, b,   k, d, s, rows, arrow) {
  nv = 0; rows = 0; d = 0; s = a
  for (k = a; k <= b; k++) {
    if (k < b && kind[k] == "open") { d++; continue }
    if (k < b && kind[k] == "close") { d--; continue }
    if (k < b && !(d == 0 && kind[k] == "punct" && text[k] == ",")) continue
    if (k > s) {
      arrow = top_find(s, k, "=>")
      if (!arrow || !lookup_pattern(s, arrow) || !lookup_value(arrow + 1, k)) return -1
      nv++; vs[nv] = arrow + 1; ve[nv] = k; rows++
    }
    s = k + 1
  }
  return rows
}
# Marks the values of each exhaustive lookup in logic as data, as the size counter does (verify::size
# `lookups`): a guard-free `match` of two or more arms, and an `enum_map!`, whose every arm maps a
# lookup_pattern to a lookup_value. Only the values: a pattern's number is still a magic literal.
function lookups(   a, o, e, k) {
  for (a = 1; a <= n; a++) {
    if (region[a] != "logic" || kind[a] != "word") continue
    if (text[a] == "match") {
      o = top_find(a + 1, n + 1, "{")
      if (!o || kind[o] != "open") continue
      e = after_group(o)
      if (lookup_rows(o + 1, e - 1) < 2) continue
    } else if (text[a] == "enum_map" && text[a + 1] == "!" && kind[a + 2] == "open") {
      e = after_group(a + 2)
      if (lookup_rows(a + 3, e - 1) < 0) continue
    } else continue
    for (k = 1; k <= nv; k++) mark(vs[k], ve[k], "data")
  }
}
# A lint-control attribute's findings: the attribute spans a..b-1.
function lint_attr(a, b, intest,   k, e, m, lint, why) {
  for (k = a; k < b; k++) {
    if (kind[k] != "word" || text[k] !~ /^(allow|expect|warn)$/ || text[k + 1] != "(") continue
    e = after_group(k + 1); why = 0
    if (text[k] == "allow") finding("lint-attr", sprintf("line %d: allow (the workspace denies clippy::allow_attributes)", lineof[k]))
    for (m = k + 2; m < e - 1; m++) {
      if (text[m] == "reason" && text[m + 1] == "=") { why = 1; m += 2; continue }
      if (kind[m] != "word") continue
      lint = text[m]
      if (text[m + 1] == "::" && kind[m + 2] == "word") { lint = lint "::" text[m + 2]; m += 2 }
      if (!intest && (lint in refused))
        finding("lint-attr", sprintf("line %d: %s(%s) suppresses a lint the workspace denies", lineof[k], text[k], lint))
    }
    if (!why) finding("lint-attr", sprintf("line %d: %s without reason = \"...\"", lineof[k], text[k]))
    k = e - 1
  }
}
# The variant imports of the `use` at a, each a finding: the enum's name and what it takes.
function variant_imports(a,   e, k, t, g, m, d, names) {
  e = a
  while (e <= n && !(kind[e] == "punct" && text[e] == ";")) {
    if (kind[e] == "open") { e = after_group(e); continue }
    e++
  }
  for (k = a + 1; k < e; k++) {
    if (kind[k] != "word" || text[k] !~ /^[A-Z]/ || text[k] == "Self" || text[k + 1] != "::") continue
    t = k + 2; names = ""
    if (text[t] == "*") names = "*"
    else if (kind[t] == "word") names = text[t]
    else if (kind[t] == "open" && text[t] == "{") {
      g = after_group(t); d = 0
      for (m = t + 1; m < g - 1; m++) {
        if (kind[m] == "open") { d++; continue }
        if (kind[m] == "close") { d--; continue }
        if (d || kind[m] != "word" || text[m] == "self" || text[m] == "as" || text[m - 1] == "as") continue
        names = names (names == "" ? "" : ", ") text[m]
      }
      if (names != "") names = "{" names "}"
    }
    if (names != "") finding("variant-import", sprintf("line %d: %s::%s (name a variant through its enum)", lineof[k], text[k], names))
  }
}
function zero_or_one(t,   v) {
  v = t; gsub(/_/, "", v)
  if (v ~ /^0[xob]/) { v = substr(v, 3); sub(/[iu](8|16|32|64|128|size)$/, "", v); return v ~ /^0*1?$/ && v != "" }
  sub(/([iu](8|16|32|64|128|size)|f32|f64)$/, "", v)
  return v ~ /^0*[01]?(\.0*)?$/ && v ~ /[0-9]/
}
function lint_file(test,   a, p, e, d, eq, k) {
  a = 1; p = 0
  while (a <= n) {
    if (text[a] == "#" && text[a + 1] == "[" && text[a + 2] == "cfg" && text[a + 3] == "(" && text[a + 4] == "test" \
        && text[a + 5] == ")" && text[a + 6] == "]") {
      e = after_item(a + 7); mark(a, e, "test"); p = e - 1; a = e; continue
    }
    if (text[a] == "#" && (text[a + 1] == "[" || (text[a + 1] == "!" && text[a + 2] == "["))) {
      e = after_group(a + (text[a + 1] == "!" ? 2 : 1)); mark(a, e, "attr"); p = e - 1; a = e; continue
    }
    if (kind[a] == "word" && ends_item(p) && text[a] ~ /^(struct|enum|union|type)$/) {
      e = after_item(a); mark(a, e, "types"); p = e - 1; a = e; continue
    }
    if (kind[a] == "word" && ends_item(p) && (text[a] == "const" || text[a] == "static") \
        && text[a + 1] !~ /^(fn|unsafe|extern)$/) {
      d = 0; eq = 0
      for (k = a; k <= n; k++) {
        if (kind[k] == "open") d++
        else if (kind[k] == "close") d--
        else if (text[k] == "=" && d == 0 && !eq) eq = k
        else if (text[k] == ";" && d == 0) break
      }
      if (eq) {
        mark(a, eq + 1, "types")
        mark(eq + 1, k, behaviour(eq + 1, k) ? "logic" : "data")
        region[k] = "types"; p = k; a = k + 1; continue
      }
    }
    region[a] = "logic"; p = a; a++
  }
  lookups()
  for (a = 1; a <= n; a++) {
    if (text[a] == "#" && (text[a + 1] == "[" || (text[a + 1] == "!" && text[a + 2] == "["))) {
      e = after_group(a + (text[a + 1] == "!" ? 2 : 1))
      lint_attr(a, e, test || region[a] == "test")
      a = e - 1; continue
    }
    if (!test && region[a] != "test" && kind[a] == "word" && text[a] == "use" && text[a + 1] != "<") variant_imports(a)
    if (test || region[a] != "logic" || kind[a] != "num" || zero_or_one(text[a])) continue
    if (text[a - 1] == ";" && text[a + 1] == "]") continue
    finding("magic-literal", sprintf("line %d: %s", lineof[a], text[a]))
  }
}
