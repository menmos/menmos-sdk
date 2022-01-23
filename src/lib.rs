pub mod fs;
mod typing;

pub use typing::FileMetadata;
use typing::*;

use std::sync::Arc;

use snafu::prelude::*;
use snafu::Whatever;

use menmos_client::Client;

type Result<T> = std::result::Result<T, Whatever>;

/// The menmos client.
pub struct Menmos {
    /// The filesystem interface to menmos.
    ///
    /// This interface should be used when manipulating concepts that are similar to files and folders.
    pub fs: fs::MenmosFs,

    client: ClientRC,
}

impl Menmos {
    pub async fn new(profile: &str) -> Result<Self> {
        let client = Client::builder()
            .with_metadata_detection()
            .with_profile(profile)
            .build()
            .await
            .with_whatever_context(|e| format!("failed to build client: {e}"))?;

        let client_rc = Arc::new(client);
        let fs = fs::MenmosFs::new(client_rc.clone());

        Ok(Self {
            fs,
            client: client_rc,
        })
    }

    /// Get a reference to the internal low-level menmos client.
    pub fn client(&self) -> &Client {
        self.client.as_ref()
    }
}
