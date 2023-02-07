load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

def rust_deps():
    rules_rust_dependencies()
    rust_register_toolchains(
        edition = "2021",
        versions = [
            "1.67.0",
            "nightly/2022-09-28",
        ],
    )
