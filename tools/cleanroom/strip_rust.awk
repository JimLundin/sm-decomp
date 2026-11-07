# strip_rust.awk: Rust source with comments removed, one output line per input line.
# String and raw-string contents are blanked to "" unless -v keep=1, so a word inside a string
# or a comment never matches a pattern; char literals are blanked either way. This is a lexer for
# the shapes Rust source has, not a parser: it is for code_check.sh.
function isid(ch) { return ch ~ /^[A-Za-z0-9_]$/ }
BEGIN { st = 0; depth = 0; rhash = 0; hashes = "" }
{
  line = $0; n = length(line); out = ""; i = 1
  while (i <= n) {
    c = substr(line, i, 1); d = substr(line, i, 2)
    if (st == 2) {                        # block comment, nestable
      if (d == "/*") { depth++; i += 2 }
      else if (d == "*/") { depth--; i += 2; if (depth == 0) { st = 0; out = out " " } }
      else i++
      continue
    }
    if (st == 1) {                        # string
      if (c == "\\") { if (keep) out = out substr(line, i, 2); i += 2; continue }
      if (c == "\"") { out = out "\""; st = 0; i++; continue }
      if (keep) out = out c
      i++; continue
    }
    if (st == 3) {                        # raw string, ended by a quote and rhash hashes
      if (c == "\"" && substr(line, i + 1, rhash) == hashes) { out = out "\""; st = 0; i += 1 + rhash; continue }
      if (keep) out = out c
      i++; continue
    }
    if (d == "//") break
    if (d == "/*") { st = 2; depth = 1; i += 2; continue }
    if (c == "\"") { st = 1; out = out c; i++; continue }
    if (c == "r") {
      p1 = (i > 1) ? substr(line, i - 1, 1) : ""
      p2 = (i > 2) ? substr(line, i - 2, 1) : ""
      if (p1 == "" || !isid(p1) || (p1 == "b" && !isid(p2))) {
        rest = substr(line, i + 1)
        if (match(rest, /^#*"/)) {
          hashes = substr(rest, 1, RLENGTH - 1); rhash = length(hashes); st = 3
          out = out "\""; i += 1 + RLENGTH; continue
        }
      }
    }
    if (c == "'") {
      if (substr(line, i + 1, 1) == "\\") {
        j = index(substr(line, i + 3), "'")
        if (j > 0) { out = out "''"; i += 3 + j; continue }
      } else if (substr(line, i + 2, 1) == "'") { out = out "''"; i += 3; continue }
    }
    out = out c; i++
  }
  print out
}
