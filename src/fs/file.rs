use menmos_client::{Meta, Type};

use snafu::prelude::*;

use crate::{ClientRC, FileMetadata, Result};

fn make_file_meta(m: FileMetadata) -> Meta {
    Meta {
        name: m.name,
        blob_type: Type::File,
        metadata: m.metadata,
        tags: m.tags,
        parents: m.parents,
        size: m.size,
    }
}

pub struct MenmosFile {
    blob_id: String,
    client: ClientRC,
}

impl MenmosFile {
    pub async fn create(client: ClientRC, metadata: FileMetadata) -> Result<Self> {
        let metadata = make_file_meta(metadata);

        let blob_id = client
            .create_empty(metadata)
            .await
            .with_whatever_context(|e| format!("failed to create file: {e}"))?;

        Ok(Self { blob_id, client })
    }
}
