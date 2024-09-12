#include "blobstore/blobstore.h"
#include <algorithm>
#include <functional>
#include <set>
#include <string>
#include <unordered_map>
#include "blobstore-rs/src/bridge.rs.h"

#include <folly/experimental/coro/BlockingWait.h>
#include <folly/experimental/coro/Collect.h>
#include <folly/futures/Future.h>
#include <folly/init/Init.h>
#include <folly/logging/xlog.h>
#include "rust/cxx_async_folly.h"

namespace org::blobstore {

class BlobstoreClient::Impl {
 private:
  friend BlobstoreClient;
  using Blob = struct {
    std::string data;
    std::set<std::string> tags;
  };
  std::map<uint64_t, Blob> blobs_;
  std::mutex mtx_;

 public:
  size_t insert_blob(std::string&& contents) {
    auto blobid = std::hash<std::string>{}(contents);
    std::lock_guard<std::mutex> lock(mtx_);
    blobs_.emplace(blobid, Blob{std::move(contents), {}});
    return blobid;
  }
};

BlobstoreClient::BlobstoreClient() : impl_(new BlobstoreClient::Impl) {}

void BlobstoreClient::tag(uint64_t blobid, rust::Str tag) const {
  impl_->blobs_[blobid].tags.emplace(tag);
}

BlobMetadata BlobstoreClient::metadata(uint64_t blobid) const {
  BlobMetadata metadata{};
  auto blob = impl_->blobs_.find(blobid);
  if (blob != impl_->blobs_.end()) {
    metadata.size = blob->second.data.size();
    std::for_each(
        blob->second.tags.cbegin(), blob->second.tags.cend(), [&](auto& t) {
          metadata.tags.emplace_back(t);
        });
  }
  return metadata;
}

std::shared_ptr<BlobstoreClient> new_blobstore_client() {
  int argc = 0;
  char** argv = 0;
FOLLY_PUSH_WARNING
FOLLY_CLANG_DISABLE_WARNING("-Wdeprecated-declarations")
  folly::init(&argc, &argv);
FOLLY_POP_WARNING
  return std::make_shared<BlobstoreClient>();
}

folly::coro::Task<uint64_t>
    BlobstoreClient::put_coro(MultiBuf& buf) const {
  std::string contents;
  while (true) {
    auto chunk = next_chunk(buf);
    if (chunk.empty()) {
      break;
    }
    contents.append(std::begin(chunk), std::end(chunk));
  }
  auto blob_id = impl_->insert_blob(std::move(contents));
  XLOGF(INFO, "blobid: {}", blob_id);
  co_return blob_id;
}

void put_coro(
    const std::shared_ptr<BlobstoreClient>& client,
    MultiBuf& buf,
    rust::Fn<void(rust::Box<BlobTx> ctx, std::uint64_t result)> ok,
    rust::Fn<void(rust::Box<BlobTx> ctx, const std::string& exn)> fail,
    rust::Box<BlobTx> ctx) noexcept {
  client->put_coro(buf)
      .semi()
      .via(folly::getGlobalCPUExecutor())
      .thenTry(
          [ok, fail, context = std::move(ctx)](auto res) mutable {
            if (res.hasValue()) {
              (*ok)(std::move(context), res.value());
            } else {
              (*fail)(std::move(context),
              res.exception().what().toStdString());
            }
          });
}

uint64_t BlobstoreClient::put(MultiBuf& buf) const {
  return folly::coro::blockingWait(
      put_coro(buf).scheduleOn(folly::getGlobalCPUExecutor()));
}

RustFutureU64 put_async(
    const std::shared_ptr<BlobstoreClient>& client, MultiBuf& buf) {
  co_return co_await client->put_coro(buf);
}

} // namespace org::blobstore
