mod file;
use file::MenmosFile;

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
}
