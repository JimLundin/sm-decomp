# shape_scan.awk: code_check.sh's crate-wide shape rules over one clean crate's library sources,
# each already stripped by strip_rust.awk and preceded by a line "\001FILE PATH", PATH relative
# to the crate's src/. The caller leaves test files out (tests/, tests.rs); #[cfg(test)] items
# are skipped here. Prints one "TAG<TAB>PATH<TAB>DETAIL" line per finding. A token lexer, not a
# parser.
#
# argument-object  (G6) a struct taken by exactly one signature and built only in the argument
#                  lists of calls to that function, with no impl (no methods, no trait), and not
#                  a field's or a variant's type, a return type, a constant's or an alias's type
#                  or named in a trait: a bag of one call's arguments. Pass the values, or give
#                  the concept a role.
function isid(ch) { return ch ~ /^[A-Za-z0-9_]$/ }
function add(k, t) { n++; kind[n] = k; text[n] = t; lineof[n] = FNR - fileline; fileof[n] = file }
BEGIN {
  nops = split("<<= >>= ... ..= :: -> => == != <= >= && || += -= *= /= %= ^= &= |= << >> ..", ops, " ")
  n = 0
}
/^\001FILE / {
  file = substr($0, 7); fileline = FNR
  next
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
    if (isid(c)) {
      j = i; while (j <= len && isid(substr(line, j, 1))) j++
      add(c ~ /[0-9]/ ? "num" : "word", substr(line, i, j - i)); i = j; continue
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
# The index after the `>` closing the `<` at a.
function after_angles(a,   d) {
  d = 0
  for (; a <= n; a++) {
    if (text[a] == "<") d++
    else if (text[a] == ">") d--
    else if (text[a] == ">>") d -= 2
    if (d <= 0) return a + 1
  }
  return a
}
# Marks the struct names among the words a..b-1 as used in role `role`.
function uses(a, b, role,   k) {
  for (k = a; k < b; k++) if (kind[k] == "word" && (text[k] in declared)) used[role, text[k]] = 1
}
# Whether a struct literal or tuple constructor at k (its name) is an expression.
function built(k, e,   p) {
  p = text[k - 1]
  if (p !~ /^([({[,;:=&!]|=>|return|mut|else)$/ && kind[k - 1] != "close") return 0
  if (p == "&" && text[k - 2] == "->") return 0
  if (p == "mut" && text[k - 2] == "&" && text[k - 3] == "->") return 0
  return text[e] !~ /^(=|=>|\|)$/
}
# The name of the function whose call's argument list holds, as one whole argument, the
# expression from token a to b-1; "" if it is not such an argument.
function call_of(a, b,   k, d) {
  if (text[a - 1] == "&") a--
  else if (text[a - 1] == "mut" && text[a - 2] == "&") a -= 2
  if (text[a - 1] != "(" && text[a - 1] != ",") return ""
  if (text[b] != "," && text[b] != ")") return ""
  d = 0
  for (k = a - 1; k >= 1; k--) {
    if (kind[k] == "close") d++
    else if (kind[k] == "open") { if (d == 0) break; d-- }
  }
  if (k < 1 || text[k] != "(" || kind[k - 1] != "word" || text[k - 1] !~ /^[a-z_]/) return ""
  if (text[k - 1] ~ /^(if|while|match|return|in|for|let|fn)$/ || text[k - 2] == "fn") return ""
  return text[k - 1]
}
END {
  # Test items are not the crate's shape.
  for (a = 1; a <= n; a++) {
    if (text[a] == "#" && text[a + 1] == "[" && text[a + 2] == "cfg" && text[a + 3] == "(" && text[a + 4] == "test" \
        && text[a + 5] == ")" && text[a + 6] == "]") {
      e = after_item(a + 7); for (k = a; k < e; k++) skip[k] = 1; a = e - 1
    }
  }
  for (a = 1; a <= n; a++) {
    if (skip[a] || text[a] != "struct" || kind[a + 1] != "word") continue
    name = text[a + 1]; declared[name] = 1; dfile[name] = fileof[a]; dline[name] = lineof[a]
    j = a + 2; if (text[j] == "<") j = after_angles(j)
    tuple[name] = text[j] == "("
  }
  for (a = 1; a <= n; a++) {
    if (skip[a]) continue
    w = text[a]
    if (kind[a] == "word" && w ~ /^(struct|enum|union|type)$/ && kind[a + 1] == "word" && text[a - 1] != "::") {
      e = after_item(a); uses(a + 2, e, "field"); a = e - 1; continue
    }
    if (kind[a] == "word" && w == "trait" && kind[a + 1] == "word") {
      e = after_item(a); uses(a + 2, e, "trait"); a = e - 1; continue
    }
    if (kind[a] == "word" && (w == "const" || w == "static") && kind[a + 1] == "word" && text[a + 1] != "fn") {
      for (j = a + 1; j <= n && text[j] != "=" && text[j] != ";"; j++);
      uses(a + 1, j, "const"); continue
    }
    if (kind[a] == "word" && w == "impl") {
      for (j = a + 1; j <= n && text[j] != "{" && text[j] != ";"; j++);
      uses(a + 1, j, "impl"); continue
    }
    if (kind[a] == "word" && w == "fn" && kind[a + 1] == "word") {
      j = a + 2; if (text[j] == "<") j = after_angles(j)
      if (text[j] != "(") continue
      e = after_group(j); split("", seen)
      for (k = j + 1; k < e - 1; k++) {
        if (kind[k] != "word" || !(text[k] in declared) || (text[k] in seen)) continue
        seen[text[k]] = 1; takers[text[k]]++; taker[text[k]] = text[a + 1]
      }
      for (k = e; k <= n && text[k] != "{" && text[k] != ";"; k++)
        if (text[k] == "->") { r = k + 1; while (r <= n && text[r] != "{" && text[r] != ";" && text[r] != "where") r++; uses(k + 1, r, "return"); break }
      continue
    }
    if (kind[a] == "word" && (w in declared) && ((text[a + 1] == "{" && !tuple[w]) || (text[a + 1] == "(" && tuple[w]))) {
      e = after_group(a + 1)
      if (!built(a, e)) continue
      f = call_of(a, e)
      if (f == "") others[w]++
      else { calls[w]++; callee[w, f] = 1 }
    }
  }
  for (s in declared) {
    if (takers[s] != 1 || calls[s] < 1 || others[s] > 0 || !((s, taker[s]) in callee)) continue
    if (("field", s) in used || ("return", s) in used || ("impl", s) in used || ("trait", s) in used || ("const", s) in used) continue
    bad = 0
    for (key in callee) { split(key, kf, SUBSEP); if (kf[1] == s && kf[2] != taker[s]) bad = 1 }
    if (bad) continue
    printf "argument-object\t%s\tline %d: struct %s is taken only by fn %s and built only in its arguments (%d calls): pass the values, or give it a role\n", \
      dfile[s], dline[s], s, taker[s], calls[s]
  }
}
