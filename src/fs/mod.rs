mod dir;
use dir::MenmosDirectory;

mod file;
use file::MenmosFile;

mod util;

use futures::TryStreamExt;

use menmos_client::Type;

use snafu::prelude::*;

use crate::{ClientRC, FileMetadata, Result};

use self::dir::DirEntry;

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

    async fn remove_blob_unchecked<S: AsRef<str>>(&self, id: S) -> Result<()> {
        // TODO: Update the menmos client so that Client::delete takes a ref.
        self.client
            .delete(String::from(id.as_ref()))
            .await
            .with_whatever_context(|e| format!("Failed to delete: {}", e))
    }

    pub async fn remove_file<S: AsRef<str>>(&self, id: S) -> Result<()> {
        match util::get_meta_if_exists(&self.client, id.as_ref()).await? {
            Some(meta) => {
                if meta.blob_type == Type::Directory {
                    whatever!("can't delete blob: is directory");
                }
                self.remove_blob_unchecked(id).await
            }
            None => Ok(()),
        }
    }

    pub async fn create_dir(&self, metadata: FileMetadata) -> Result<MenmosDirectory> {
        MenmosDirectory::create(self.client.clone(), metadata).await
    }

    pub async fn remove_dir<S: AsRef<str>>(&self, id: S) -> Result<()> {
        match util::get_meta_if_exists(&self.client, id.as_ref()).await? {
            Some(meta) => {
                if meta.blob_type == Type::File {
                    whatever!("can't delete blob: is file");
                }

                let dir = MenmosDirectory::open_raw(self.client.clone(), id.as_ref(), meta)?;

                if !dir.is_empty().await? {
                    whatever!("cannot delete: directory is not empty");
                }
                self.remove_blob_unchecked(id).await
            }
            None => Ok(()),
        }
    }

    pub async fn remove_dir_all<S: AsRef<str>>(&self, id: S) -> Result<()> {
        match util::get_meta_if_exists(&self.client, id.as_ref()).await? {
            Some(meta) => {
                if meta.blob_type == Type::File {
                    whatever!("can't delete blob: is file");
                }

                let dir = MenmosDirectory::open_raw(self.client.clone(), id.as_ref(), meta)?;

                // We don't do the deletion recursively because recursivity + async requires a lot of indirection.
                let mut delete_stack: Vec<DirEntry> = vec![DirEntry::Directory(dir)];
                while let Some(target) = delete_stack.pop() {
                    match target {
                        DirEntry::File(file) => self.remove_blob_unchecked(file.id()).await?,
                        DirEntry::Directory(dir) => {
                            let children = dir.list().try_collect::<Vec<_>>().await?;

                            delete_stack.extend(children.into_iter());
                            self.remove_blob_unchecked(dir.id()).await?;
                        }
                    }
                }

                Ok(())
            }

            None => Ok(()),
        }
    }
}
