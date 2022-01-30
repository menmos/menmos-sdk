use std::collections::HashMap;
use std::path::{Path, PathBuf};

use menmos_client::{Meta, Type};

use snafu::prelude::*;

use crate::error;
use crate::metadata_detector::MetadataDetectorRC;
use crate::ClientRC;

#[derive(Debug, Snafu)]
pub enum PushError {
    MetadataPopulationError {
        source: error::MetadataDetectorError,
    },
    // TODO: add source: ClientError once its exposed in menmos-client >= 0.1.0
    #[snafu(display("failed to push '{:?}'", path))]
    BlobPushError { path: PathBuf },
}

type Result<T> = std::result::Result<T, PushError>;

pub struct PushResult {
    pub source_path: PathBuf,
    pub blob_id: String,
    pub parent_id: Option<String>,
}

pub(crate) async fn push_file<P: AsRef<Path>>(
    path: P,
    client: ClientRC,
    metadata_detector: &MetadataDetectorRC,
    tags: Vec<String>,
    meta_map: HashMap<String, String>,
    blob_type: Type,
    parent: Option<String>,
) -> Result<String> {
    let mut meta = Meta::new(
        path.as_ref()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        blob_type.clone(),
    );

    metadata_detector
        .populate(path.as_ref(), &mut meta)
        .context(MetadataPopulationSnafu)?;

    if blob_type == Type::File {
        meta = meta.with_size(path.as_ref().metadata().unwrap().len())
    }

    if let Some(parent) = parent {
        meta = meta.with_parent(parent);
    }

    for tag in tags.iter() {
        meta = meta.with_tag(tag);
    }

    for (k, v) in meta_map.iter() {
        meta = meta.with_meta(k, v);
    }

    let item_id = client
        .push(path.as_ref(), meta)
        .await
        .map_err(|_| PushError::BlobPushError {
            path: PathBuf::from(path.as_ref()),
        })?;

    Ok(item_id)
}
