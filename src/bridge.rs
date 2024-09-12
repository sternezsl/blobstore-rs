use crate::next_chunk;
use crate::BlobTx;
use std::future::Future;

#[cxx::bridge(namespace=org::blobstore)]
pub mod ffi {
    struct BlobMetadata {
        size: usize,
        tags: Vec<String>,
    }

    struct VecU8 {
        value: Vec<u8>,
    }

    struct MultiBuf {
        chunks: Vec<VecU8>,
        pos: usize,
    }

    extern "Rust" {
        type BlobTx; // opaque type for C++
        fn next_chunk(buf: &mut MultiBuf) -> &[u8];
    }

    unsafe extern "C++" {
        include!("blobstore/blobstore.h");
        type BlobstoreClient; // opaque type for Rust
        fn new_blobstore_client() -> SharedPtr<BlobstoreClient>;
        fn put(&self, parts: &mut MultiBuf) -> u64;
        fn tag(&self, blobid: u64, tag: &str);
        fn metadata(&self, blobid: u64) -> BlobMetadata;
        fn put_coro(
            client: &SharedPtr<BlobstoreClient>,
            arg: &mut MultiBuf,
            ok: fn(Box<BlobTx>, ret: u64),
            fail: fn(Box<BlobTx>, ret: &CxxString),
            tx: Box<BlobTx>,
        );
    }

    unsafe extern "C++" {
        type RustFutureU64 = crate::bridge::RustFutureU64;
        fn put_async(client: &SharedPtr<BlobstoreClient>, arg: &mut MultiBuf) -> RustFutureU64;
    }
}

#[cxx_async::bridge(namespace=org::blobstore)]
unsafe impl Future for RustFutureU64 {
    type Output = u64;
}
