load("@rules_rust//rust:repositories.bzl", "rust_repositories")
load("//third_party/rust/crates:crates.bzl", "raze_fetch_remote_crates")

def fetch_remote_crates():
    raze_fetch_remote_crates()

def rust_deps():
    rust_repositories(
        edition = "2021",
        version = "1.60.0",
    )
    fetch_remote_crates()
