use std::io::SeekFrom;

use futures::TryStreamExt;

use menmos::*;

#[tokio::test]
async fn menmos_file_api() -> std::result::Result<(), Box<dyn std::error::Error>> {
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

#[tokio::test]
async fn menmos_dir_api() -> Result<(), Box<dyn std::error::Error>> {
    let client = Menmos::new("local").await?;

    let dir_a = client
        .fs
        .create_dir(FileMetadata::new("dir_a").with_tag("sdk_test"))
        .await?;

    let dir_b = client
        .fs
        .create_dir(FileMetadata::new("dir_b").with_tag("sdk_test"))
        .await?;

    client
        .fs
        .create_file(
            FileMetadata::new("file_a")
                .with_tag("sdk_test")
                .with_parent(dir_a.id()),
        )
        .await?;

    client
        .fs
        .create_file(
            FileMetadata::new("file_b")
                .with_tag("sdk_test")
                .with_parent(dir_b.id()),
        )
        .await?;

    let results = dir_a.list().try_collect::<Vec<_>>().await?;
    assert_eq!(results.len(), 1);

    // TODO: Delete our files.

    Ok(())
}
