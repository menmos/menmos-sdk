use std::collections::HashMap;
use std::sync::Arc;

use menmos_client::Client;

pub type ClientRC = Arc<Client>;

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
