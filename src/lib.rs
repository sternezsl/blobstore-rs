use anyhow::{Error, Result};
mod bridge;
pub use bridge::ffi;

pub struct BlobTx(oneshot::Sender<Result<u64>>);

pub async fn put_coro(
    client: &cxx::SharedPtr<ffi::BlobstoreClient>,
    parts: &mut ffi::MultiBuf,
) -> Result<u64> {
    let (tx, rx) = oneshot::channel();
    ffi::put_coro(
        client,
        parts,
        |tx, blobid| {
            let _ = tx.0.send(Ok(blobid));
        },
        |tx, err_msg| {
            let _ = tx.0.send(Err(Error::msg(err_msg.to_string())));
        },
        Box::new(BlobTx(tx)),
    );
    rx.await?
}

pub fn next_chunk(buf: &mut ffi::MultiBuf) -> &[u8] {
    let next = buf.chunks.get(buf.pos);
    buf.pos += 1;
    next.map_or(&[], |arg| Vec::as_slice(&arg.value))
}

// pub async fn put_async(
//     client: &cxx::SharedPtr<ffi::BlobstoreClient>,
//     parts: &mut ffi::MultiBuf,
// ) -> Result<u64> {
//     Ok(bridge::ffi::put_async(client, parts).await?)
// }
