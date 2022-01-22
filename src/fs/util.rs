use interface::BlobMeta;

use snafu::prelude::*;

use crate::{ClientRC, Result};

pub async fn get_meta_if_exists(client: &ClientRC, blob_id: &str) -> Result<Option<BlobMeta>> {
    let r = client
        .get_meta(blob_id)
        .await
        .with_whatever_context(|e| format!("failed to get meta: {e}"))?;
    Ok(r)
}

pub async fn get_meta(client: &ClientRC, blob_id: &str) -> Result<BlobMeta> {
    get_meta_if_exists(client, blob_id)
        .await?
        .with_whatever_context(|| "missing blob meta")
}
