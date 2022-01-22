mod dir;
use dir::MenmosDirectory;

mod file;
use file::MenmosFile;

mod util;

use menmos_client::Type;
use snafu::prelude::*;

use crate::{ClientRC, FileMetadata, Result};

pub struct MenmosFs {
    client: ClientRC,
}

impl MenmosFs {
    pub fn new(client: ClientRC) -> Self {
        Self { client }
    }

    pub async fn create_file(&self, metadata: FileMetadata) -> Result<MenmosFile> {
        MenmosFile::create(self.client.clone(), metadata).await
    }

    pub async fn remove_file<S: AsRef<str>>(&self, id: S) -> Result<()> {
        match util::get_meta_if_exists(&self.client, id.as_ref()).await? {
            Some(meta) => {
                if meta.blob_type == Type::Directory {
                    whatever!("can't delete blob: is directory");
                }
                self.client
                    .delete(String::from(id.as_ref()))
                    .await
                    .with_whatever_context(|e| format!("failed to delete: {e}"))?;
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub async fn create_dir(&self, metadata: FileMetadata) -> Result<MenmosDirectory> {
        MenmosDirectory::create(self.client.clone(), metadata).await
    }
}
