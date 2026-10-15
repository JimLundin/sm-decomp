//! Private verification scope: source reuse and bounded input accounting.
//! Generic IO lives in tool-execution; the operation-local typed cache lives
//! in decomp-source and is inherited alongside the same worker input budget.
#[allow(unused_imports)]
pub(crate) use decomp_source::context::{
    SourceContext as SharedScope, discovery, discovery_with_limit, limited, reconcile, shared_scope, until,
};
#[cfg(test)]
pub(crate) use tool_execution::bounded::charged_bytes;
#[allow(unused_imports)]
pub(crate) use tool_execution::bounded::{
    DEFAULT_BYTES, MAX_DECLARED_PATHS, MAX_DOCUMENT_BYTES, MAX_FILE_BYTES, account, check, count, digest, digest_limit,
    entries, immutable_copy, now_unix_ms, object, read, read_limit, read_with_digest, safe, text, tree, write,
};
#[cfg(test)]
pub(crate) mod tests {
    pub(crate) use decomp_source::testing::{append_tar, gzip_archive};
    pub(crate) use tool_execution::testing::Directory;
}
