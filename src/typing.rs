use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use menmos_client::Client;

pub type ClientRC = Arc<Client>;

/// The metadata of a blob.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileMetadata {
    /// The name of this file/folder. Does not need to be unique.
    pub name: String,

    /// The key/value pairs for this file.
    pub metadata: HashMap<String, String>,

    /// The tags for this file.
    pub tags: Vec<String>,

    /// This file's parent IDs.
    pub parents: Vec<String>,

    /// This file's size, in bytes.
    pub size: u64,
}

impl FileMetadata {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn with_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tags.push(tag.into());
        self
    }

    #[must_use]
    pub fn with_meta<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    #[must_use]
    pub fn with_parent<P: Into<String>>(mut self, parent: P) -> Self {
        self.parents.push(parent.into());
        self
    }

    #[must_use]
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UploadRequest {
    /// The path of the file to upload.
    pub path: PathBuf,

    pub metadata: HashMap<String, String>,

    pub tags: Vec<String>,

    pub parent_id: Option<String>,
}
