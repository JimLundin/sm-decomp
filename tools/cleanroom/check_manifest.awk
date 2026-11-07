# check_manifest.awk: the findings in one clean crate's Cargo.toml, one `tag<TAB>detail` per line.
#   -v crate=ABS_CRATE_DIR  -v allowed=PATH|PATH  (the directories a path dependency may name)
#   -v allowfile=allowed_deps.txt  (lines `name version kind`, `#` comments)
# Not a TOML parser: it reads the manifest shapes cargo's own templates and ours produce, and
# reports a dependency line it can't read as a finding.
function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t\r]+$/, "", s); return s }
function unq(s) { s = trim(s); if (s ~ /^".*"$/ || s ~ /^'.*'$/) s = substr(s, 2, length(s) - 2); return s }
function emit(tag, detail) { print tag "\t" detail }
function norm(p,    n, parts, k, stack, top, res) {
  n = split(p, parts, "/"); top = 0
  for (k = 1; k <= n; k++) {
    if (parts[k] == "" || parts[k] == ".") continue
    if (parts[k] == "..") { if (top > 0) top--; continue }
    stack[++top] = parts[k]
  }
  res = ""
  for (k = 1; k <= top; k++) res = res "/" stack[k]
  return res
}
function record(kind, name, key, val) { seen[kind SUBSEP name] = 1; d[kind SUBSEP name SUBSEP key] = val }
BEGIN {
  while ((getline l < allowfile) > 0) {
    sub(/#.*/, "", l); l = trim(l)
    if (l == "") continue
    split(l, f, /[ \t]+/); allowv[f[1]] = f[2]; allowk[f[1]] = f[3]
  }
  np = split(allowed, ap, "|")
  for (k = 1; k <= np; k++) okpath[norm(ap[k])] = 1
  sec = ""; mode = ""
}
{
  line = $0; sub(/\r$/, "", line)
  t = trim(line)
  if (t == "" || t ~ /^#/) next
  if (t ~ /^\[/) {
    sec = t; sub(/^\[+/, "", sec); sub(/\][ \t]*(#.*)?$/, "", sec); sub(/\]+$/, "", sec); sec = trim(sec)
    secn = sec; gsub(/"[^"]*"|'[^']*'/, "Q", secn)
    mode = ""
    if (secn ~ /(^|\.)build-dependencies(\.|$)/) emit("build-deps", "[" sec "]")
    if (secn ~ /^(patch|replace|workspace)(\.|$)/) emit("manifest-section", "[" sec "]")
    if (secn ~ /(^|\.)(dev-)?dependencies$/) {
      mode = "deps"; kind = (secn ~ /dev-dependencies$/) ? "dev" : "normal"
    } else if (secn ~ /(^|\.)(dev-)?dependencies\.[^.]+$/) {
      mode = "deptable"; kind = (secn ~ /dev-dependencies\.[^.]+$/) ? "dev" : "normal"
      tname = sec; sub(/^.*dependencies\./, "", tname); tname = unq(tname)
    } else if (secn ~ /^(lib|bin|test|example|bench)$/) mode = "target"
    else if (secn == "package") mode = "package"
    next
  }
  if (t !~ /=/) next
  eq = index(t, "="); k = trim(substr(t, 1, eq - 1)); v = trim(substr(t, eq + 1))
  if (k ~ /^proc[-_]macro$/) emit("proc-macro", t)
  if (mode == "package" && k == "build") emit("build-key", t)
  if (mode == "package" && k == "links") emit("links-key", t)
  if (mode == "target" && k == "path") {
    pv = unq(v)
    if (pv ~ /^\// || pv ~ /(^|\/)\.\.(\/|$)/ || pv ~ /^[A-Za-z]:/) emit("target-path", t)
  }
  if (mode == "deptable") { record(kind, tname, k, unq(v)); next }
  if (mode != "deps") next
  dot = index(k, ".")
  if (dot > 0) { record(kind, unq(substr(k, 1, dot - 1)), unq(substr(k, dot + 1)), unq(v)); next }
  name = unq(k)
  if (v ~ /^["']/) { record(kind, name, "version", unq(v)); next }
  if (v ~ /^\{/) {
    rest = v; got = 0
    while (match(rest, /[A-Za-z_-]+[ \t]*=[ \t]*("[^"]*"|'[^']*'|true|false|\[[^]]*\])/)) {
      kv = substr(rest, RSTART, RLENGTH); rest = substr(rest, RSTART + RLENGTH)
      e = index(kv, "="); record(kind, name, trim(substr(kv, 1, e - 1)), unq(substr(kv, e + 1))); got = 1
    }
    if (!got) record(kind, name, "unreadable", v)
    seen[kind SUBSEP name] = 1
    next
  }
  record(kind, name, "unreadable", v)
}
END {
  for (nk in seen) {
    split(nk, a, SUBSEP); kind = a[1]; name = a[2]; pre = kind SUBSEP name SUBSEP
    real = ((pre "package") in d) ? d[pre "package"] : name
    label = name; if (real != name) label = name " (package " real ")"
    if ((pre "unreadable") in d) { emit("dep-registry", label ": unreadable dependency line `" d[pre "unreadable"] "`"); continue }
    if ((pre "git") in d || (pre "registry") in d || (pre "registry-index") in d) {
      emit("dep-git", label ": git or alternate-registry dependency"); continue
    }
    if ((pre "workspace") in d) { emit("dep-workspace", label ": inherited from a workspace manifest, which this check can't see"); continue }
    if ((pre "path") in d) {
      p = d[pre "path"]; r = (p ~ /^\//) ? norm(p) : norm(crate "/" p)
      if (!(r in okpath)) emit("dep-path", label ": path " p " resolves outside the clean crates (" r ")")
      continue
    }
    if (!(real in allowv)) { emit("dep-registry", label ": not in allowed_deps.txt"); continue }
    ver = ((pre "version") in d) ? d[pre "version"] : ""
    if (ver != allowv[real] || kind != allowk[real])
      emit("dep-registry", label ": allowed only as a " allowk[real] " dependency pinned `" allowv[real] "`, found " kind " `" ver "`")
  }
}
