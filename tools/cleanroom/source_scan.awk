# source_scan.awk: code_check.sh's per-source rules over one crate's sources in a single process,
# run from the crate's directory as
#   awk -v denied="..." -f strip_rust.awk -f lint_scan.awk -f source_scan.awk FILE...
# with each FILE a path under src/ or tests/. Prints, file by file in argument order, one
# "FILE<TAB>TAG<TAB>DETAIL" line per finding: lint_scan.awk's first, then the rules below in the
# order code_check.sh lists them. A `?include` line carries an include!/include_str! call
# (DETAIL is the call and up to 200 characters after it) for code_check.sh to resolve, since
# whether its path is allowed depends on where the file and the export root sit on disk.
#
# Files, the network, processes, the environment, include_bytes!, env! and assembly are clippy's
# (tools/cleanroom/clippy.toml), and unsafe code, extern blocks and link attributes the
# workspace's forbidden `unsafe_code`. What stays here is what a lint can't say: which file an
# include! may read, and the renames that would hide an include! from this scan (`std as x`,
# `std::$name`, `core::include_str as x`), and #[path = ...].
#
# The rules match over the source flattened to one line, as code_check.sh documents: comments
# removed, string contents blanked (except for include calls, which need their literal), every
# run of whitespace one space, and no space around `::` and the brackets and punctuation
# ]{}(),;!#*[, so `std ::\n fs` reads `std::fs`. Each rule takes its matches left to right
# without overlap, a match being the leftmost and then the longest, as `grep -o` takes them.
# POSIX awk has no word boundary, so the rules that need one check the characters around a match.

function wordc(ch) { return ch != "" && ch ~ /^[A-Za-z0-9_]$/ }
# Whether a match from i to j-1 starts and ends a word, as `\b` asks before a word character and
# after one.
function starts_word(s, i) { return i == 1 || !wordc(substr(s, i - 1, 1)) }
function ends_word(s, j) { return !wordc(substr(s, j, 1)) }

# The next match of RE in S at or after FROM, needing a word boundary before it (BS) or after it
# (BE): 1 with the match at MS for ML characters, or 0. A match failing a boundary is retried one
# character on, as a regex engine would move on; none of the rules has a shorter alternative that
# could meet the boundary where the longest failed.
function next_match(s, from, re, bs, be,   t) {
  while (from <= length(s)) {
    t = substr(s, from)
    if (!match(t, re)) return 0
    MS = from + RSTART - 1; ML = RLENGTH
    if ((!bs || starts_word(s, MS)) && (!be || ends_word(s, MS + ML))) return 1
    from = MS + 1
  }
  return 0
}

# The source flattened, as above, from its stripped lines joined by spaces.
function squash(s,   out, p) {
  gsub(/[\r\t]/, " ", s); gsub(/ +/, " ", s)
  out = ""
  while (match(s, / ?(::|[{}(),;!#*]|\[|\]) ?/)) {
    p = substr(s, RSTART, RLENGTH); gsub(/ /, "", p)
    out = out substr(s, 1, RSTART - 1) p
    s = substr(s, RSTART + RLENGTH)
  }
  return out s
}

function finding(tag, detail) { printf "%s\t%s\t%s\n", FILE_PATH, tag, detail }

# Each rule F(s) reports every match of its pattern in the flattened source s.
function each(s, re, bs, be, tag, detail,   from, m) {
  from = 1
  while (next_match(s, from, re, bs, be)) {
    m = substr(s, MS, ML)
    finding(tag == "" ? m : tag, detail == "" ? m : detail)
    from = MS + ML
  }
}

# include!/include_str! calls in the source with its strings kept, for code_check.sh.
function includes(k,   from, len) {
  from = 1
  while (next_match(k, from, "include(_str)?![([{]", 1, 0)) {
    len = ML + 200; if (MS + len - 1 > length(k)) len = length(k) - MS + 1
    finding("?include", substr(k, MS, len))
    from = MS + len
  }
}

# The start of the next attribute at or after FROM (`#[` or `#![`), with AO at its `[` and AE at
# the `]` that ends its first bracket (past the end if none): 0 if there is none.
function next_attr(s, from,   h, e) {
  while ((h = index(substr(s, from), "#")) > 0) {
    h += from - 1
    if (substr(s, h + 1, 1) == "[") AO = h + 1
    else if (substr(s, h + 1, 2) == "![") AO = h + 2
    else { from = h + 1; continue }
    e = index(substr(s, AO + 1), "]"); AE = e ? AO + e : length(s) + 1
    return h
  }
  return 0
}

# The end (one past) of the last TEXT in the attribute's first bracket that starts a word, or 0.
function last_in_attr(s, text,   q, n, best) {
  best = 0; n = length(text)
  for (q = AO + 1; q + n - 1 < AE; q++)
    if (substr(s, q, n) == text && starts_word(s, q)) best = q + n
  return best
}

# #[path = ...] (`#!?\[[^]]*\bpath ?=`): the attribute up to its last `path =` or `path=`.
function path_attrs(s,   from, h, end) {
  from = 1
  for (h = next_attr(s, from); h; h = next_attr(s, from)) {
    end = last_in_attr(s, "path =")
    if (last_in_attr(s, "path=") > end) end = last_in_attr(s, "path=")
    if (end) { finding("path-attr", substr(s, h, end - h)); from = end }
    else from = h + 1
  }
}

function scan(b, k) {
  each(b, "std as [A-Za-z_]+", 1, 0, "std-alias", "")
  each(b, "std::\\{([^;}]*,)?self as ", 0, 0, "std-alias", "")
  each(b, "std::\\$[A-Za-z_]+", 0, 0, "std-macro-ident", "")
  each(b, "(std|core|alloc)::(include_str|include)", 0, 1, "macro-path", "")
  includes(k)
  path_attrs(b)
}

# The driver: the files are read here, not as awk's input, so each is one unit and an empty file
# is still scanned.
BEGIN {
  # Prefixed names: lint_scan.awk's lexer keeps its working values in globals.
  for (src_argi = 1; src_argi < ARGC; src_argi++) {
    FILE_PATH = ARGV[src_argi]
    src_test = FILE_PATH ~ /^tests\// || FILE_PATH ~ /\/tests\// || FILE_PATH ~ /\/tests\.rs$/
    strip_reset(); lint_reset(); src_lineno = 0; src_b = ""; src_k = ""
    while ((getline src_raw < FILE_PATH) > 0) {
      src_line = strip_line(src_raw, 0)
      lint_line(src_line, ++src_lineno)
      src_b = src_b src_line " "; src_k = src_k strip_line(src_raw, 1) " "
    }
    close(FILE_PATH)
    lint_file(src_test)
    scan(squash(src_b), squash(src_k))
  }
}
