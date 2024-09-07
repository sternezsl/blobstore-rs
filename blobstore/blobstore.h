#pragma once
#include <memory>
#include <folly/coro/Task.h>
#include "rust/cxx.h"

struct BlobMetadata;
struct MultiBuf;
struct BlobTx;

class BlobstoreClient {
 public:
  BlobstoreClient();
  uint64_t put(MultiBuf& buf) const;
  void tag(uint64_t blobid, rust::Str tag) const;
  BlobMetadata metadata(uint64_t blobid) const;
  folly::coro::Task<uint64_t> put_coro(MultiBuf& buf) const;

 private:
  class Impl;
  std::shared_ptr<Impl> impl_;
};

std::shared_ptr<BlobstoreClient> new_blobstore_client();

void put_coro(
    const std::shared_ptr<BlobstoreClient>& client,
    MultiBuf& arg,
    rust::Fn<void(rust::Box<BlobTx> ctx, std::uint64_t ret)> ok,
    rust::Fn<void(rust::Box<BlobTx> ctx, const std::string& exn)> fail,
    rust::Box<BlobTx> ctx) noexcept;
