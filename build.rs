fn main() {
    let folly_include_dir =
        std::env::var("FOLLY_INCLUDE_DIR").expect("Can't read FOLLY_INCLUDE_DIR env var");
    let mut builder = cxx_build::bridge("src/bridge.rs");
    builder
        .file("blobstore/blobstore.cc")
        .std("c++20")
        .include("./")
        .flag("-DFMT_HEADER_ONLY")
        .flag("-DUSE_FOLLY_LOGGING")
        .flag("-Wno-nullability-completeness")
        .flag("-Wno-deprecated-builtins")
        .include(&folly_include_dir);
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if std::env::var_os("CARGO_CFG_UNIX").is_some() {
        builder
            .flag("-D_GNU_SOURCE=1")
            .flag("-finput-charset=UTF-8")
            .flag("-fsigned-char")
            .flag("-faligned-new");
        if let Some(target_os) = std::env::var_os("CARGO_CFG_TARGET_OS") {
            if target_os == "linux" {
                builder
                    .flag("-DHAVE_LINUX_TIME_TYPES_H")
                    .include(&format!("{}/libevent-1.4.14b", folly_include_dir))
                    .include(&format!(
                        "{}/libevent-1.4.14b/{}-linux",
                        folly_include_dir, target_arch
                    ));
            } else if target_os == "macos" && target_arch == "aarch64" {
                builder
                    .include(&format!("{}/libevent-2.1.12", folly_include_dir))
                    .include(&format!(
                        "{}/libevent-2.1.12/arm64-darwin",
                        folly_include_dir
                    ));
            }
        }
    } else if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        builder
            .flag("-DBOOST_CONFIG_SUPPRESS_OUTDATED_MESSAGE")
            .flag("-DFOLLY_INTERNAL_NONSTDC_NAMES")
            .flag("-DFOLLY_MINIGLOG_USING_SHARED_LIBRARY")
            .flag("-DBOOST_ALL_NO_LIB")
            .flag("-Wno-unsafe-buffer-usage")
            .flag("-Wno-dollar-in-identifier-extension")
            .flag("/EHs")
            .flag("/GF")
            .flag("/Zc:rvalueCast")
            .flag("/Zc:strictStrings")
            .flag("/Zc:threadSafeInit")
            .flag("/permissive-")
            .include(&format!("{}/libevent-2.1.12", folly_include_dir))
            .include(&format!(
                "{}/libevent-2.1.12/{}-windows",
                folly_include_dir, target_arch
            ));
    }

    builder.compile("blobstore");

    println!("cargo:rerun-if-changed=src/bridge.rs");
    println!("cargo:rerun-if-changed=blobstore/blobstore.cc");
    println!("cargo:rerun-if-changed=blobstore/blobstore.h");
}
