use std::collections::HashMap;
use std::path::{Path, PathBuf};

use menmos_client::{Meta, Type};
use snafu::ResultExt;

use crate::{ClientRC, Result};

pub struct PushResult {
    pub source_path: PathBuf,
    pub blob_id: String,
    pub parent_id: Option<String>,
}

pub(crate) async fn push_file<P: AsRef<Path>>(
    path: P,
    client: ClientRC,
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
    )
    .with_meta(
        "extension",
        path.as_ref()
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_else(String::default),
    );

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
        .with_whatever_context(|e| format!("failed to push [{:?}]: {}", path.as_ref(), e))?;

    Ok(item_id)
}
