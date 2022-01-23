use futures::TryStream;

use interface::{BlobMeta, Hit};

use menmos_client::Query;

use snafu::prelude::*;

use crate::{ClientRC, Result};

pub async fn get_meta_if_exists(client: &ClientRC, blob_id: &str) -> Result<Option<BlobMeta>> {
    let r = client
        .get_meta(blob_id)
        .await
        .with_whatever_context(|e| format!("failed to get meta: {e}"))?;
    Ok(r)
}

pub async fn get_meta(client: &ClientRC, blob_id: &str) -> Result<BlobMeta> {
    get_meta_if_exists(client, blob_id)
        .await?
        .with_whatever_context(|| "missing blob meta")
}

/// Scrolls a given query until the end of results and returns the output lazily as a stream.
pub fn scroll_query(
    query: Query,
    client: &ClientRC,
) -> impl TryStream<Ok = Hit, Error = snafu::Whatever> {
    futures::stream::try_unfold(
        (query, Vec::<Hit>::new(), false, client.clone()),
        move |(mut n_query, mut pending_hits, mut page_end_reached, client)| async move {
            if let Some(hit) = pending_hits.pop() {
                return Ok(Some((
                    hit,
                    (n_query, pending_hits, page_end_reached, client),
                )));
            }

            if page_end_reached {
                return Ok(None);
            }

            let client = client.clone();

            let results = client
                .query(n_query.clone())
                .await
                .with_whatever_context(|e| format!("query failed: {e}"))?;

            pending_hits.extend(results.hits.into_iter());

            n_query.from += results.count;
            page_end_reached = n_query.from >= results.total;

            let r_val = pending_hits.pop().unwrap();
            let ret_tuple = (n_query, pending_hits, page_end_reached, client);
            Ok(Some((r_val, ret_tuple)))
        },
    )
}
