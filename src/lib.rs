pub mod fs;
mod metadata_detector;
mod profile;
pub mod push;
mod typing;
mod util;

pub use profile::{Config, Profile};
pub use typing::FileMetadata;

use metadata_detector::MetadataDetector;
use typing::*;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time;

use async_stream::try_stream;

use futures::TryStream;

use menmos_client::{Client, Type};

use snafu::prelude::*;
use snafu::Whatever;

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

    metadata_detector: MetadataDetector,
}

impl Menmos {
    fn new_with_client(client: Client) -> Self {
        let client_rc = Arc::new(client);
        let fs = fs::MenmosFs::new(client_rc.clone());

        // If this fails we shipped a bad library.
        let metadata_detector = MetadataDetector::new().unwrap();

        Self {
            fs,
            client: client_rc,
            metadata_detector,
        }
    }

    pub async fn new(profile: &str) -> Result<Self> {
        let profile = load_profile_from_config(profile)?;
        let client = Client::builder()
            .with_host(profile.host)
            .with_username(profile.username)
            .with_password(profile.password)
            .build()
            .await
            .with_whatever_context(|e| format!("failed to build client: {e}"))?;
        Ok(Self::new_with_client(client))
    }

    /// Get a builder to configure the client.
    pub fn builder(profile: &str) -> MenmosBuilder {
        MenmosBuilder::new(profile.into())
    }

    /// Get a reference to the internal low-level menmos client.
    pub fn client(&self) -> &Client {
        let v: Vec<PathBuf> = Vec::new();
        self.client.as_ref()
    }

    /// Recursively push a sequence of files and/or directories to the menmos cluster.
    pub fn push_files(
        &self,
        paths: Vec<PathBuf>,
        tags: Vec<String>,
        metadata: HashMap<String, String>,
        parent_id: Option<String>,
    ) -> impl TryStream<Ok = push::PushResult, Error = snafu::Whatever> {
        let client = self.client.clone();

        try_stream! {
            let mut working_stack = Vec::new();
            working_stack.extend(paths.into_iter().map(|path| (parent_id.clone(), path)));

            while let Some((parent_maybe, file_path)) = working_stack.pop(){
                if file_path.is_file() {
                    let blob_id = push::push_file(file_path.clone(), client.clone(), tags.clone(), metadata.clone(), Type::File, parent_maybe.clone()).await?;
                    yield push::PushResult{source_path: file_path, blob_id, parent_id: parent_maybe.clone()};
                } else {
                    let directory_id: String = push::push_file(
                        file_path.clone(),
                        client.clone(),
                        tags.clone(),
                        metadata.clone(),
                        Type::Directory,
                        parent_maybe,
                    )
                    .await?;

                    // Add this directory's children to the working stack.
                    let read_dir_result: Result<std::fs::ReadDir> = file_path.read_dir().with_whatever_context(|e| format!("failed to read directory: {e}"));
                    for child in read_dir_result?.filter_map(|f| f.ok()) {
                        working_stack.push((Some(directory_id.clone()), child.path()));
                    }
                }
            }
        }
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
