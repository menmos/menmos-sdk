use futures::{TryStream, TryStreamExt};

use menmos_client::{Meta, Query, Type};

use snafu::prelude::*;

use crate::{ClientRC, FileMetadata, Result};

use super::file::MenmosFile;
use super::util;

fn make_dir_meta(m: FileMetadata) -> Meta {
    Meta {
        name: m.name,
        blob_type: Type::Directory,
        metadata: m.metadata,
        tags: m.tags,
        parents: m.parents,
        size: m.size,
    }
}

#[derive(Clone)]
pub enum DirEntry {
    File(MenmosFile),
    Directory(MenmosDirectory),
}

#[derive(Clone)]
pub struct MenmosDirectory {
    blob_id: String,
    client: ClientRC,
}

impl MenmosDirectory {
    pub async fn create(client: ClientRC, metadata: FileMetadata) -> Result<Self> {
        let metadata = make_dir_meta(metadata);

        let blob_id = client
            .create_empty(metadata)
            .await
            .with_whatever_context(|e| format!("failed to create directory: {e}"))?;

        Ok(Self { blob_id, client })
    }

    pub async fn open(client: ClientRC, id: &str) -> Result<Self> {
        let metadata = util::get_meta(&client, id).await?;
        if metadata.blob_type == Type::File {
            whatever!("is file");
        }

        Ok(Self {
            blob_id: String::from(id),
            client,
        })
    }

    pub fn id(&self) -> &str {
        &self.blob_id
    }

    pub fn list(&self) -> impl TryStream<Ok = DirEntry, Error = snafu::Whatever> + Unpin {
        let query = Query::default()
            .and_parent(&self.blob_id)
            .with_from(0)
            .with_size(50);

        let client = self.client.clone();
        Box::pin(util::scroll_query(query, &client).and_then(move |hit| {
            let client = client.clone();
            async move {
                let entry = if hit.meta.blob_type == Type::File {
                    DirEntry::File(MenmosFile::open(client, &hit.id).await?)
                } else {
                    DirEntry::Directory(MenmosDirectory::open(client, &hit.id).await?)
                };
                Ok(entry)
            }
        }))
    }
}
