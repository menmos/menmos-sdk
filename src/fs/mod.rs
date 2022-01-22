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
        let meta = self
            .client
            .get_meta(id.as_ref())
            .await
            .with_whatever_context(|e| format!("failed to get file meta: {e}"))?;

        match meta {
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
}
