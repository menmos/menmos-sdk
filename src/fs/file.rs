use std::io::SeekFrom;

use bytes::Bytes;

use menmos_client::{Meta, Type};

use snafu::prelude::*;

use super::util;
use crate::{ClientRC, FileMetadata, Result};

fn make_file_meta(m: FileMetadata) -> Meta {
    Meta {
        name: m.name,
        blob_type: Type::File,
        metadata: m.metadata,
        tags: m.tags,
        parents: m.parents,
        size: m.size,
    }
}

pub struct MenmosFile {
    blob_id: String,
    client: ClientRC,
    offset: u64,
}

impl MenmosFile {
    pub async fn create(client: ClientRC, metadata: FileMetadata) -> Result<Self> {
        let metadata = make_file_meta(metadata);

        let blob_id = client
            .create_empty(metadata)
            .await
            .with_whatever_context(|e| format!("failed to create file: {e}"))?;

        Ok(Self {
            blob_id,
            client,
            offset: 0,
        })
    }

    pub async fn open(client: ClientRC, id: &str) -> Result<Self> {
        let metadata = client
            .get_meta(id)
            .await
            .with_whatever_context(|e| format!("failed to fetch the file metadata: {}", e))?;

        match metadata {
            Some(metadata) => {
                if metadata.blob_type == Type::Directory {
                    whatever!("is directory");
                }
                Ok(Self {
                    blob_id: String::from(id),
                    client,
                    offset: 0,
                })
            }
            None => {
                whatever!("file not found");
            }
        }
    }

    pub fn id(&self) -> &str {
        &self.blob_id
    }

    pub async fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let buf = Bytes::copy_from_slice(buf);
        let buf_len = buf.len();
        self.client
            .write(&self.blob_id, self.offset, buf)
            .await
            .with_whatever_context(|e| format!("failed to write to file: {e}"))?;
        self.offset += buf_len as u64;
        Ok(buf_len)
    }

    pub async fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            SeekFrom::Current(offset) => {
                let new_offset = self.offset as i64 + offset;
                if new_offset < 0 {
                    whatever!("seek reached negative offset");
                }
                self.offset = new_offset as u64;
            }
            SeekFrom::Start(new_offset) => {
                self.offset = new_offset;
            }
            SeekFrom::End(relative) => {
                let metadata = util::get_meta(&self.client, &self.blob_id).await?;
                let end_offset = metadata.size as i64;
                let new_offset = end_offset + relative;
                if new_offset < 0 {
                    whatever!("seek reached negative offset");
                }
                self.offset = new_offset as u64;
            }
        }
        Ok(self.offset)
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let r = self
            .client
            .read_range(
                &self.blob_id,
                (self.offset, (self.offset + buf.len() as u64) - 1),
            )
            .await
            .with_whatever_context(|e| format!("failed to read from file: {e}"))?;
        buf.copy_from_slice(&r);
        self.offset += r.len() as u64;
        Ok(r.len())
    }

    pub async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let metadata = util::get_meta(&self.client, &self.blob_id).await?;
        let out = self
            .client
            .read_range(&self.blob_id, (self.offset, metadata.size))
            .await
            .with_whatever_context(|e| format!("failed to read from file: {e}"))?;
        *buf = out;
        self.offset += buf.len() as u64;
        Ok(buf.len())
    }

    pub async fn read_to_string(&mut self, string: &mut String) -> Result<usize> {
        let mut v = Vec::new();
        self.read_to_end(&mut v).await?;

        let buf_read = v.len();

        *string =
            String::from_utf8(v).with_whatever_context(|_| "buffer value is not valid UTF-8")?;

        Ok(buf_read)
    }
}
