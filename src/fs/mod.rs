//! The filesystem SDK module.

mod dir;
pub use dir::{DirEntry, MenmosDirectory};

mod file;
pub use file::MenmosFile;

use futures::TryStreamExt;

use menmos_client::Type;

use snafu::prelude::*;

use crate::{ClientRC, FileMetadata, Result};

/// The entrypoint structure of the filesystem SDK.
#[derive(Clone)]
pub struct MenmosFs {
    client: ClientRC,
}

impl MenmosFs {
    #[doc(hidden)]
    pub fn new(client: ClientRC) -> Self {
        Self { client }
    }

    /// Create a new file with the provided metadata.
    ///
    /// This function will return a handle to the created file, at offset 0.
    ///
    /// # Examples
    /// ```no_run
    /// use menmos::FileMetadata;
    /// # use menmos::fs::MenmosFs;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let client = menmos_client::Client::new("a", "b", "c").await.unwrap();
    /// # let fs = MenmosFs::new(std::sync::Arc::new(client));
    /// let handle = fs.create_file(FileMetadata::new("test.txt").with_tag("sdk_file"))
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
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

    /// Remove a file by its ID.
    ///
    /// If the specified blob ID does not exist, no error is returned and no operation
    /// is performed.
    ///
    /// # Errors
    ///
    /// If this function is called with an ID corresponding to a blob that is _not_
    /// a file, an error variant will be returned.
    ///
    /// # Examples
    /// ```no_run
    /// # use menmos::fs::MenmosFs;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let client = menmos_client::Client::new("a", "b", "c").await.unwrap();
    /// # let fs = MenmosFs::new(std::sync::Arc::new(client));
    /// fs.remove_file("<a file blob ID>").await.unwrap();
    /// # }
    /// ```
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

    /// Create an empty directory with the provided metadata.
    ///
    /// This function will return a handle to the created directory.
    /// # Examples
    /// ```no_run
    /// use menmos::FileMetadata;
    /// # use menmos::fs::MenmosFs;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let client = menmos_client::Client::new("a", "b", "c").await.unwrap();
    /// # let fs = MenmosFs::new(std::sync::Arc::new(client));
    /// let handle = fs.create_dir(FileMetadata::new("my_directory").with_tag("sdk_dir"))
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn create_dir(&self, metadata: FileMetadata) -> Result<MenmosDirectory> {
        MenmosDirectory::create(self.client.clone(), metadata).await
    }

    /// Remove a directory by its ID.
    ///
    /// If the specified blob ID does not exist, no error is returned and no operation
    /// is performed.
    ///
    /// # Errors
    ///
    /// If this function is called with an ID corresponding to a blob that is _not_
    /// a directory, an error variant will be returned.
    ///
    /// If this function is called with an ID corresponding to a directory that is _not_
    /// empty, an error variant will also be returned.
    /// # Examples
    /// ```no_run
    /// # use menmos::fs::MenmosFs;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let client = menmos_client::Client::new("a", "b", "c").await.unwrap();
    /// # let fs = MenmosFs::new(std::sync::Arc::new(client));
    /// fs.remove_dir("<a dir blob ID>").await.unwrap();
    /// # }
    /// ```
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

    /// Recursively remove a directory along with all its children.
    ///
    /// If the specified blob ID does not exist, no error is returned and no operation
    /// is performed.
    ///
    /// # Errors
    ///
    /// If this function is called with an ID corresponding to a blob that is _not_
    /// a directory, an error variant will be returned.
    ///
    /// # Examples
    /// ```no_run
    /// # use menmos::fs::MenmosFs;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let client = menmos_client::Client::new("a", "b", "c").await.unwrap();
    /// # let fs = MenmosFs::new(std::sync::Arc::new(client));
    /// fs.remove_dir_all("<a dir blob ID>").await.unwrap();
    /// # }
    /// ```
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
