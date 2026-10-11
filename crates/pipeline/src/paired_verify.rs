use anyhow::{Result, ensure};
use std::path::Path;
use toml::Value;
pub fn verify_files(cache: &Path, focus: &Value) -> Result<()> {
    let pair = pipeline_model::PairData::decode(&std::fs::read(cache.join("pair.bin"))?)?;
    let validation: pipeline_model::PairValidation = pipeline_model::read(&cache.join("validation.toml"))?;
    let record: pipeline_model::CacheRecord = pipeline_model::read(&cache.join("cache.toml"))?;
    ensure!(
        record.schema_version == 2
            && record.format == "sm-paired-cache-v2"
            && Some(record.start_input_index as u64)
                == focus["start_input_index"].as_integer().and_then(|n| u64::try_from(n).ok())
            && Some(record.start_instruction as u64)
                == focus["start_instruction"].as_integer().and_then(|n| u64::try_from(n).ok())
            && record.pair_file_sha256 == crate::digest(&cache.join("pair.bin"))?
            && record.validation_file_sha256 == crate::digest(&cache.join("validation.toml"))?,
        "paired cache files differ"
    );
    ensure!(
        pair.pair_sha256 == validation.pair_sha256
            && pair.metadata.identities == validation.identities
            && focus["cache_pair_sha256"]
                == crate::readiness_metadata::value(&pair.pair_sha256).expect("validated native metadata")
            && focus["cache_history_sha256"]
                == crate::readiness_metadata::value(&pair.metadata.history_sha256).expect("validated native metadata"),
        "paired validation/history identity differs"
    );
    if let Some(candidate) = pair.metadata.identities.settings.candidate_sha256.as_deref() {
        ensure!(Some(candidate) == focus["binary_sha256"].as_str(), "paired cache candidate differs from focus");
    }
    crate::readiness_contract::validate("checkpoint-validation", &Value::try_from(validation)?)?;
    Ok(())
}
