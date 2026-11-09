# vendor_nobuild_cfg.awk -v name=NAME -v pred=PREDICATE FILE: FILE with the cfg NAME replaced by
# PREDICATE wherever it stands as a cfg option: inside `cfg(...)`, `cfg!(...)` and the condition
# of `cfg_attr(..., ...)`, across lines. A module, path or string of the same name is kept.
# Not a Rust parser: it reads the predicate shapes rustc accepts, without comments in them.
function word(c) { return c ~ /[A-Za-z0-9_]/ }
function subst(p,    r, k, before, after) {
  r = ""
  while ((k = index(p, name)) > 0) {
    before = (k > 1) ? substr(p, k - 1, 1) : ""
    after = substr(p, k + length(name), 1)
    if (word(before) || before == "\"" || word(after) || after == "\"") r = r substr(p, 1, k - 1 + length(name))
    else r = r substr(p, 1, k - 1) pred
    p = substr(p, k + length(name))
  }
  return r p
}
{ buf = buf $0 "\n" }
END {
  n = length(buf); i = 1; out = ""
  while (i <= n) {
    rest = substr(buf, i)
    if (!match(rest, /cfg(_attr)?!?[ \t\n]*\(/)) { out = out rest; break }
    s = i + RSTART - 1
    out = out substr(rest, 1, RSTART - 1 + RLENGTH)
    i = s + RLENGTH
    if (s > 1 && word(substr(buf, s - 1, 1))) continue
    attr = (substr(buf, s, 8) == "cfg_attr")
    depth = 1; j = i
    while (j <= n) {
      c = substr(buf, j, 1)
      if (c == "(") depth++
      else if (c == ")") { if (--depth == 0) break }
      else if (c == "," && depth == 1 && attr) break
      j++
    }
    out = out subst(substr(buf, i, j - i))
    i = j
  }
  printf "%s", out
}
