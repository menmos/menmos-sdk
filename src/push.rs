use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_stream::try_stream;

use futures::TryStream;

use crate::ClientRC;

pub struct PushResult {
    pub source_path: PathBuf,
    pub blob_id: String,
}

pub fn recursive_push(
    paths: &[Path],
    tags: Vec<String>,
    metadata: HashMap<String, String>,
    parent_id: Option<String>,
    client: ClientRC,
) -> impl TryStream<Ok = PushResult, Error = snafu::Whatever> {
    // TODO: Rewrite this from the menmos-cli code.
    unimplemented!()
}
