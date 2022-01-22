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
    use std::io::SeekFrom;

    use super::*;

    #[tokio::test]
    async fn test_fs() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let client = Menmos::new("local").await?;

        // Test file creation
        let mut file = client
            .fs
            .create_file(FileMetadata::new("test.txt").with_tag("sdk_test"))
            .await?;

        // Test writing to the file.
        file.write("Hello world!".as_bytes()).await?;

        // Seek to beginning
        file.seek(SeekFrom::Start(0)).await?;

        // Read the first word.
        let mut buf = Vec::new();
        buf.resize(5, 0_u8);
        let read = file.read(&mut buf).await?;

        assert_eq!(read, 5);
        assert_eq!(&String::from_utf8(buf)?, "Hello");

        // After reading "hello" we should be at the space in the file. Seek by one to hit the second word.
        file.seek(SeekFrom::Current(1)).await?;
        // Overwrite "world" with "there".
        file.write("there".as_bytes()).await?;

        // Seek back to the beginning.
        file.seek(SeekFrom::Start(0)).await?;

        // Read everything back.
        let mut full_buf = Vec::new();
        file.read_to_end(&mut full_buf).await?;

        assert_eq!(&String::from_utf8(full_buf)?, "Hello there!");

        // Go to the beginning again.
        file.seek(SeekFrom::Start(0)).await?;

        // Test string reading.
        let mut buf = String::new();
        file.read_to_string(&mut buf).await?;
        assert_eq!(&buf, "Hello there!");

        // Delete.
        client.fs.remove_file(file.id()).await?;

        Ok(())
    }
}
