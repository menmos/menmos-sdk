pub mod fs;
mod profile;
mod typing;

pub use profile::{Config, Profile};

pub use typing::FileMetadata;
use typing::*;

use std::sync::Arc;
use std::time;

use snafu::prelude::*;
use snafu::Whatever;

use menmos_client::Client;

type Result<T> = std::result::Result<T, Whatever>;

fn load_profile_from_config(profile: &str) -> Result<Profile> {
    let config = Config::load()?;
    config
        .profiles
        .get(profile)
        .cloned()
        .with_whatever_context(|| format!("missing profile: {profile}"))
}

/// The menmos client.
#[derive(Clone)]
pub struct Menmos {
    /// The filesystem interface to menmos.
    ///
    /// This interface should be used when manipulating concepts that are similar to files and folders.
    pub fs: fs::MenmosFs,

    client: ClientRC,
}

impl Menmos {
    fn new_with_client(client: Client) -> Self {
        let client_rc = Arc::new(client);
        let fs = fs::MenmosFs::new(client_rc.clone());

        Self {
            fs,
            client: client_rc,
        }
    }

    pub async fn new(profile: &str) -> Result<Self> {
        let profile = load_profile_from_config(profile)?;
        let client = Client::builder()
            .with_host(profile.host)
            .with_username(profile.username)
            .with_password(profile.password)
            .with_metadata_detection()
            .build()
            .await
            .with_whatever_context(|e| format!("failed to build client: {e}"))?;
        Ok(Self::new_with_client(client))
    }

    pub fn builder(profile: &str) -> MenmosBuilder {
        MenmosBuilder::new(profile.into())
    }

    /// Get a reference to the internal low-level menmos client.
    pub fn client(&self) -> &Client {
        self.client.as_ref()
    }
}

pub struct MenmosBuilder {
    profile: String,
    request_timeout: Option<time::Duration>,
    max_retry_count: Option<usize>,
    retry_interval: Option<time::Duration>,
}

impl MenmosBuilder {
    pub(crate) fn new(profile: String) -> Self {
        Self {
            profile,
            request_timeout: None,
            max_retry_count: None,
            retry_interval: None,
        }
    }

    #[must_use]
    pub fn with_request_timeout(mut self, request_timeout: time::Duration) -> Self {
        self.request_timeout = Some(request_timeout);
        self
    }

    #[must_use]
    pub fn with_max_retry_count(mut self, max_retry_count: usize) -> Self {
        self.max_retry_count = Some(max_retry_count);
        self
    }

    #[must_use]
    pub fn with_retry_interval(mut self, retry_interval: time::Duration) -> Self {
        self.retry_interval = Some(retry_interval);
        self
    }

    pub async fn build(self) -> Result<Menmos> {
        let profile = load_profile_from_config(&self.profile)?;
        let mut builder = Client::builder()
            .with_metadata_detection()
            .with_host(profile.host)
            .with_username(profile.username)
            .with_password(profile.password);

        if let Some(request_timeout) = self.request_timeout {
            builder = builder.with_request_timeout(request_timeout);
        }

        if let Some(max_retry_count) = self.max_retry_count {
            builder = builder.with_max_retry_count(max_retry_count);
        }

        if let Some(retry_interval) = self.retry_interval {
            builder = builder.with_retry_interval(retry_interval);
        }

        let client = builder
            .build()
            .await
            .with_whatever_context(|e| format!("failed to build client: {e}"))?;

        Ok(Menmos::new_with_client(client))
    }
}
