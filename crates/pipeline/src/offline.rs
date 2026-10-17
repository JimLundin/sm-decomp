//! Application composition of Offline verification and compiled host identity.
use anyhow::Result;
use offline_verification::{HostInputs, OfflineFamilies};

pub(crate) fn host_inputs() -> Result<HostInputs> {
    HostInputs::new(crate::source_stamps::HOST_SOURCES, crate::binary_digest()?.to_owned())
}

pub(crate) fn families() -> Result<OfflineFamilies> {
    Ok(OfflineFamilies::new(host_inputs()?))
}
