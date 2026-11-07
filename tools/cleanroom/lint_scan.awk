# lint_scan.awk: code_check.sh's style rules over one source, already stripped by strip_rust.awk
# (comments gone, strings blanked to "", char literals to ''). Prints one "TAG<TAB>DETAIL" line
# per finding. A token lexer and the size counter's regions, not a parser.
#   -v denied="..."  lints the workspace denies by name (and its denied groups), space-separated
#   -v test=1        the whole file is test code (a tests/ directory or a tests.rs file)
#
# lint-attr      (L1) an #[allow] (the workspace denies clippy::allow_attributes), an
#                #[expect]/#[allow]/#[warn] without reason = "...", or, outside test code, one
#                that names a lint in `denied`. A lint denied only through a group (pedantic,
#                nursery) may be expected, with its reason.
# magic-literal  (L2) a number other than 0 or 1 in logic. Exempt: test code, attributes,
#                struct/enum/union/type items, a constant's head, a constant's value unless it
#                holds behaviour (the counter's L3), an array length (`[T; N]`, `[x; N]`) and a
#                tuple field (`.2`).
function isid(ch) { return ch ~ /^[A-Za-z0-9_]$/ }
function add(k, t) { n++; kind[n] = k; text[n] = t; lineof[n] = NR }
BEGIN {
  nops = split("<<= >>= ... ..= :: -> => == != <= >= && || += -= *= /= %= ^= &= |= << >> ..", ops, " ")
  nd = split(denied, dl, " ")
  for (i = 1; i <= nd; i++) refused[dl[i]] = 1
  n = 0
}
{
  line = $0; len = length(line); i = 1
  while (i <= len) {
    c = substr(line, i, 1)
    if (c ~ /[ \t\r]/) { i++; continue }
    if (c == "\"") { j = index(substr(line, i + 1), "\""); add("lit", "\"\""); i += (j > 0 ? j + 1 : len); continue }
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
# Whether the tokens a..b-1 hold behaviour, as the size counter's L3 decides.
function behaviour(a, b,   k, w, p) {
  for (k = a; k < b; k++) {
    w = text[k]; p = k > a ? k - 1 : 0
    if (kind[k] == "word") {
      if (w ~ /^(if|match|loop|while|for|let|fn|move)$/) return 1
      if (w ~ /^[a-z_]/ && w !~ /^(mut|as|in|return|ref|dyn|impl|const|unsafe|crate|super|self)$/ \
          && ((kind[k + 1] == "open" && text[k + 1] == "(") || (text[k + 1] == "::" && text[k + 2] == "<"))) return 1
    } else if (kind[k] == "punct") {
      if (w == "!" && p && kind[p] == "word") return 1
      if ((w == "|" || w == "||") && (!p || kind[p] == "open" || text[p] ~ /^(,|;|:|=>)$/)) return 1
    }
  }
  return 0
}
function mark(a, b, r,   k) { for (k = a; k < b; k++) region[k] = r }
# A lint-control attribute's findings: the attribute spans a..b-1.
function lint_attr(a, b, intest,   k, e, m, lint, why) {
  for (k = a; k < b; k++) {
    if (kind[k] != "word" || text[k] !~ /^(allow|expect|warn)$/ || text[k + 1] != "(") continue
    e = after_group(k + 1); why = 0
    if (text[k] == "allow") printf "lint-attr\tline %d: allow (the workspace denies clippy::allow_attributes)\n", lineof[k]
    for (m = k + 2; m < e - 1; m++) {
      if (text[m] == "reason" && text[m + 1] == "=") { why = 1; m += 2; continue }
      if (kind[m] != "word") continue
      lint = text[m]
      if (text[m + 1] == "::" && kind[m + 2] == "word") { lint = lint "::" text[m + 2]; m += 2 }
      if (!intest && (lint in refused))
        printf "lint-attr\tline %d: %s(%s) suppresses a lint the workspace denies\n", lineof[k], text[k], lint
    }
    if (!why) printf "lint-attr\tline %d: %s without reason = \"...\"\n", lineof[k], text[k]
    k = e - 1
  }
}
function zero_or_one(t,   v) {
  v = t; gsub(/_/, "", v)
  if (v ~ /^0[xob]/) { v = substr(v, 3); sub(/[iu](8|16|32|64|128|size)$/, "", v); return v ~ /^0*1?$/ && v != "" }
  sub(/([iu](8|16|32|64|128|size)|f32|f64)$/, "", v)
  return v ~ /^0*[01]?(\.0*)?$/ && v ~ /[0-9]/
}
END {
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
  for (a = 1; a <= n; a++) {
    if (text[a] == "#" && (text[a + 1] == "[" || (text[a + 1] == "!" && text[a + 2] == "["))) {
      e = after_group(a + (text[a + 1] == "!" ? 2 : 1))
      lint_attr(a, e, test || region[a] == "test")
      a = e - 1; continue
    }
    if (test || region[a] != "logic" || kind[a] != "num" || zero_or_one(text[a])) continue
    if (text[a - 1] == ";" && text[a + 1] == "]") continue
    printf "magic-literal\tline %d: %s\n", lineof[a], text[a]
  }
}
