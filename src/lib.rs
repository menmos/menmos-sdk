mod fs;
mod typing;

use typing::*;

use std::sync::Arc;

use snafu::prelude::*;
use snafu::Whatever;

use menmos_client::Client;

type Result<T> = std::result::Result<T, Whatever>;

pub struct Menmos {
    pub fs: fs::MenmosFs,
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
        let fs = fs::MenmosFs::new(client_rc);

        Ok(Self { fs })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let client = Menmos::new("local").await?;

        Ok(())
    }
}
