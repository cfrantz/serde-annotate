"""
@generated
cargo-raze generated Bazel file.

DO NOT EDIT! Replaced on runs of cargo-raze
"""

load("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")  # buildifier: disable=load
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")  # buildifier: disable=load
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")  # buildifier: disable=load

# EXPERIMENTAL -- MAY CHANGE AT ANY TIME: A mapping of package names to a set of normal dependencies for the Rust targets of that package.
_DEPENDENCIES = {
    "": {
        "ansi_term": "@raze__ansi_term__0_12_1//:ansi_term",
        "inventory": "@raze__inventory__0_2_3//:inventory",
        "num-traits": "@raze__num_traits__0_2_15//:num_traits",
        "once_cell": "@raze__once_cell__1_14_0//:once_cell",
        "pest": "@raze__pest__2_3_1//:pest",
        "regex": "@raze__regex__1_6_0//:regex",
        "serde": "@raze__serde__1_0_144//:serde",
        "thiserror": "@raze__thiserror__1_0_35//:thiserror",
    },
    "annotate_derive": {
        "proc-macro-error": "@raze__proc_macro_error__1_0_4//:proc_macro_error",
        "proc-macro2": "@raze__proc_macro2__1_0_43//:proc_macro2",
        "quote": "@raze__quote__1_0_21//:quote",
        "syn": "@raze__syn__1_0_100//:syn",
    },
}

# EXPERIMENTAL -- MAY CHANGE AT ANY TIME: A mapping of package names to a set of proc_macro dependencies for the Rust targets of that package.
_PROC_MACRO_DEPENDENCIES = {
    "": {
        "pest_derive": "@raze__pest_derive__2_3_1//:pest_derive",
    },
    "annotate_derive": {
    },
}

# EXPERIMENTAL -- MAY CHANGE AT ANY TIME: A mapping of package names to a set of normal dev dependencies for the Rust targets of that package.
_DEV_DEPENDENCIES = {
    "": {
        "anyhow": "@raze__anyhow__1_0_65//:anyhow",
        "clap": "@raze__clap__3_2_22//:clap",
        "deser-hjson": "@raze__deser_hjson__1_0_2//:deser_hjson",
        "json5": "@raze__json5__0_4_1//:json5",
        "serde_bytes": "@raze__serde_bytes__0_11_7//:serde_bytes",
        "serde_json": "@raze__serde_json__1_0_85//:serde_json",
        "serde_yaml": "@raze__serde_yaml__0_8_26//:serde_yaml",
    },
    "annotate_derive": {
    },
}

# EXPERIMENTAL -- MAY CHANGE AT ANY TIME: A mapping of package names to a set of proc_macro dev dependencies for the Rust targets of that package.
_DEV_PROC_MACRO_DEPENDENCIES = {
    "": {
        "serde_derive": "@raze__serde_derive__1_0_144//:serde_derive",
    },
    "annotate_derive": {
    },
}

def crate_deps(deps, package_name = None):
    """EXPERIMENTAL -- MAY CHANGE AT ANY TIME: Finds the fully qualified label of the requested crates for the package where this macro is called.

    WARNING: This macro is part of an expeirmental API and is subject to change.

    Args:
        deps (list): The desired list of crate targets.
        package_name (str, optional): The package name of the set of dependencies to look up.
            Defaults to `native.package_name()`.
    Returns:
        list: A list of labels to cargo-raze generated targets (str)
    """

    if not package_name:
        package_name = native.package_name()

    # Join both sets of dependencies
    dependencies = _flatten_dependency_maps([
        _DEPENDENCIES,
        _PROC_MACRO_DEPENDENCIES,
        _DEV_DEPENDENCIES,
        _DEV_PROC_MACRO_DEPENDENCIES,
    ])

    if not deps:
        return []

    missing_crates = []
    crate_targets = []
    for crate_target in deps:
        if crate_target not in dependencies[package_name]:
            missing_crates.append(crate_target)
        else:
            crate_targets.append(dependencies[package_name][crate_target])

    if missing_crates:
        fail("Could not find crates `{}` among dependencies of `{}`. Available dependencies were `{}`".format(
            missing_crates,
            package_name,
            dependencies[package_name],
        ))

    return crate_targets

def all_crate_deps(normal = False, normal_dev = False, proc_macro = False, proc_macro_dev = False, package_name = None):
    """EXPERIMENTAL -- MAY CHANGE AT ANY TIME: Finds the fully qualified label of all requested direct crate dependencies \
    for the package where this macro is called.

    If no parameters are set, all normal dependencies are returned. Setting any one flag will
    otherwise impact the contents of the returned list.

    Args:
        normal (bool, optional): If True, normal dependencies are included in the
            output list. Defaults to False.
        normal_dev (bool, optional): If True, normla dev dependencies will be
            included in the output list. Defaults to False.
        proc_macro (bool, optional): If True, proc_macro dependencies are included
            in the output list. Defaults to False.
        proc_macro_dev (bool, optional): If True, dev proc_macro dependencies are
            included in the output list. Defaults to False.
        package_name (str, optional): The package name of the set of dependencies to look up.
            Defaults to `native.package_name()`.

    Returns:
        list: A list of labels to cargo-raze generated targets (str)
    """

    if not package_name:
        package_name = native.package_name()

    # Determine the relevant maps to use
    all_dependency_maps = []
    if normal:
        all_dependency_maps.append(_DEPENDENCIES)
    if normal_dev:
        all_dependency_maps.append(_DEV_DEPENDENCIES)
    if proc_macro:
        all_dependency_maps.append(_PROC_MACRO_DEPENDENCIES)
    if proc_macro_dev:
        all_dependency_maps.append(_DEV_PROC_MACRO_DEPENDENCIES)

    # Default to always using normal dependencies
    if not all_dependency_maps:
        all_dependency_maps.append(_DEPENDENCIES)

    dependencies = _flatten_dependency_maps(all_dependency_maps)

    if not dependencies:
        return []

    return dependencies[package_name].values()

def _flatten_dependency_maps(all_dependency_maps):
    """Flatten a list of dependency maps into one dictionary.

    Dependency maps have the following structure:

    ```python
    DEPENDENCIES_MAP = {
        # The first key in the map is a Bazel package
        # name of the workspace this file is defined in.
        "package_name": {

            # An alias to a crate target.     # The label of the crate target the
            # Aliases are only crate names.   # alias refers to.
            "alias":                          "@full//:label",
        }
    }
    ```

    Args:
        all_dependency_maps (list): A list of dicts as described above

    Returns:
        dict: A dictionary as described above
    """
    dependencies = {}

    for dep_map in all_dependency_maps:
        for pkg_name in dep_map:
            if pkg_name not in dependencies:
                # Add a non-frozen dict to the collection of dependencies
                dependencies.setdefault(pkg_name, dict(dep_map[pkg_name].items()))
                continue

            duplicate_crate_aliases = [key for key in dependencies[pkg_name] if key in dep_map[pkg_name]]
            if duplicate_crate_aliases:
                fail("There should be no duplicate crate aliases: {}".format(duplicate_crate_aliases))

            dependencies[pkg_name].update(dep_map[pkg_name])

    return dependencies

def raze_fetch_remote_crates():
    """This function defines a collection of repos and should be called in a WORKSPACE file"""
    maybe(
        http_archive,
        name = "raze__aho_corasick__0_7_19",
        url = "https://crates.io/api/v1/crates/aho-corasick/0.7.19/download",
        type = "tar.gz",
        sha256 = "b4f55bd91a0978cbfd91c457a164bab8b4001c833b7f323132c0a4e1922dd44e",
        strip_prefix = "aho-corasick-0.7.19",
        build_file = Label("//third_party/rust/crates/remote:BUILD.aho-corasick-0.7.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ansi_term__0_12_1",
        url = "https://crates.io/api/v1/crates/ansi_term/0.12.1/download",
        type = "tar.gz",
        sha256 = "d52a9bb7ec0cf484c551830a7ce27bd20d67eac647e1befb56b0be4ee39a55d2",
        strip_prefix = "ansi_term-0.12.1",
        build_file = Label("//third_party/rust/crates/remote:BUILD.ansi_term-0.12.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__anyhow__1_0_65",
        url = "https://crates.io/api/v1/crates/anyhow/1.0.65/download",
        type = "tar.gz",
        sha256 = "98161a4e3e2184da77bb14f02184cdd111e83bbbcc9979dfee3c44b9a85f5602",
        strip_prefix = "anyhow-1.0.65",
        build_file = Label("//third_party/rust/crates/remote:BUILD.anyhow-1.0.65.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__atty__0_2_14",
        url = "https://crates.io/api/v1/crates/atty/0.2.14/download",
        type = "tar.gz",
        sha256 = "d9b39be18770d11421cdb1b9947a45dd3f37e93092cbf377614828a319d5fee8",
        strip_prefix = "atty-0.2.14",
        build_file = Label("//third_party/rust/crates/remote:BUILD.atty-0.2.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__autocfg__1_1_0",
        url = "https://crates.io/api/v1/crates/autocfg/1.1.0/download",
        type = "tar.gz",
        sha256 = "d468802bab17cbc0cc575e9b053f41e72aa36bfa6b7f55e3529ffa43161b97fa",
        strip_prefix = "autocfg-1.1.0",
        build_file = Label("//third_party/rust/crates/remote:BUILD.autocfg-1.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bitflags__1_3_2",
        url = "https://crates.io/api/v1/crates/bitflags/1.3.2/download",
        type = "tar.gz",
        sha256 = "bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a",
        strip_prefix = "bitflags-1.3.2",
        build_file = Label("//third_party/rust/crates/remote:BUILD.bitflags-1.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__block_buffer__0_10_3",
        url = "https://crates.io/api/v1/crates/block-buffer/0.10.3/download",
        type = "tar.gz",
        sha256 = "69cce20737498f97b993470a6e536b8523f0af7892a4f928cceb1ac5e52ebe7e",
        strip_prefix = "block-buffer-0.10.3",
        build_file = Label("//third_party/rust/crates/remote:BUILD.block-buffer-0.10.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cfg_if__1_0_0",
        url = "https://crates.io/api/v1/crates/cfg-if/1.0.0/download",
        type = "tar.gz",
        sha256 = "baf1de4339761588bc0619e3cbc0120ee582ebb74b53b4efbf79117bd2da40fd",
        strip_prefix = "cfg-if-1.0.0",
        build_file = Label("//third_party/rust/crates/remote:BUILD.cfg-if-1.0.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__clap__3_2_22",
        url = "https://crates.io/api/v1/crates/clap/3.2.22/download",
        type = "tar.gz",
        sha256 = "86447ad904c7fb335a790c9d7fe3d0d971dc523b8ccd1561a520de9a85302750",
        strip_prefix = "clap-3.2.22",
        build_file = Label("//third_party/rust/crates/remote:BUILD.clap-3.2.22.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__clap_derive__3_2_18",
        url = "https://crates.io/api/v1/crates/clap_derive/3.2.18/download",
        type = "tar.gz",
        sha256 = "ea0c8bce528c4be4da13ea6fead8965e95b6073585a2f05204bd8f4119f82a65",
        strip_prefix = "clap_derive-3.2.18",
        build_file = Label("//third_party/rust/crates/remote:BUILD.clap_derive-3.2.18.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__clap_lex__0_2_4",
        url = "https://crates.io/api/v1/crates/clap_lex/0.2.4/download",
        type = "tar.gz",
        sha256 = "2850f2f5a82cbf437dd5af4d49848fbdfc27c157c3d010345776f952765261c5",
        strip_prefix = "clap_lex-0.2.4",
        build_file = Label("//third_party/rust/crates/remote:BUILD.clap_lex-0.2.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cpufeatures__0_2_5",
        url = "https://crates.io/api/v1/crates/cpufeatures/0.2.5/download",
        type = "tar.gz",
        sha256 = "28d997bd5e24a5928dd43e46dc529867e207907fe0b239c3477d924f7f2ca320",
        strip_prefix = "cpufeatures-0.2.5",
        build_file = Label("//third_party/rust/crates/remote:BUILD.cpufeatures-0.2.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__crypto_common__0_1_6",
        url = "https://crates.io/api/v1/crates/crypto-common/0.1.6/download",
        type = "tar.gz",
        sha256 = "1bfb12502f3fc46cca1bb51ac28df9d618d813cdc3d2f25b9fe775a34af26bb3",
        strip_prefix = "crypto-common-0.1.6",
        build_file = Label("//third_party/rust/crates/remote:BUILD.crypto-common-0.1.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ctor__0_1_23",
        url = "https://crates.io/api/v1/crates/ctor/0.1.23/download",
        type = "tar.gz",
        sha256 = "cdffe87e1d521a10f9696f833fe502293ea446d7f256c06128293a4119bdf4cb",
        strip_prefix = "ctor-0.1.23",
        build_file = Label("//third_party/rust/crates/remote:BUILD.ctor-0.1.23.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__deser_hjson__1_0_2",
        url = "https://crates.io/api/v1/crates/deser-hjson/1.0.2/download",
        type = "tar.gz",
        sha256 = "1f486ff51f3ecdf9364736375a4b358b6eb9f02555d5324fa4837c00b5aa23f5",
        strip_prefix = "deser-hjson-1.0.2",
        build_file = Label("//third_party/rust/crates/remote:BUILD.deser-hjson-1.0.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__digest__0_10_5",
        url = "https://crates.io/api/v1/crates/digest/0.10.5/download",
        type = "tar.gz",
        sha256 = "adfbc57365a37acbd2ebf2b64d7e69bb766e2fea813521ed536f5d0520dcf86c",
        strip_prefix = "digest-0.10.5",
        build_file = Label("//third_party/rust/crates/remote:BUILD.digest-0.10.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__generic_array__0_14_6",
        url = "https://crates.io/api/v1/crates/generic-array/0.14.6/download",
        type = "tar.gz",
        sha256 = "bff49e947297f3312447abdca79f45f4738097cc82b06e72054d2223f601f1b9",
        strip_prefix = "generic-array-0.14.6",
        build_file = Label("//third_party/rust/crates/remote:BUILD.generic-array-0.14.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ghost__0_1_6",
        url = "https://crates.io/api/v1/crates/ghost/0.1.6/download",
        type = "tar.gz",
        sha256 = "eb19fe8de3ea0920d282f7b77dd4227aea6b8b999b42cdf0ca41b2472b14443a",
        strip_prefix = "ghost-0.1.6",
        build_file = Label("//third_party/rust/crates/remote:BUILD.ghost-0.1.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hashbrown__0_12_3",
        url = "https://crates.io/api/v1/crates/hashbrown/0.12.3/download",
        type = "tar.gz",
        sha256 = "8a9ee70c43aaf417c914396645a0fa852624801b24ebb7ae78fe8272889ac888",
        strip_prefix = "hashbrown-0.12.3",
        build_file = Label("//third_party/rust/crates/remote:BUILD.hashbrown-0.12.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__heck__0_4_0",
        url = "https://crates.io/api/v1/crates/heck/0.4.0/download",
        type = "tar.gz",
        sha256 = "2540771e65fc8cb83cd6e8a237f70c319bd5c29f78ed1084ba5d50eeac86f7f9",
        strip_prefix = "heck-0.4.0",
        build_file = Label("//third_party/rust/crates/remote:BUILD.heck-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hermit_abi__0_1_19",
        url = "https://crates.io/api/v1/crates/hermit-abi/0.1.19/download",
        type = "tar.gz",
        sha256 = "62b467343b94ba476dcb2500d242dadbb39557df889310ac77c5d99100aaac33",
        strip_prefix = "hermit-abi-0.1.19",
        build_file = Label("//third_party/rust/crates/remote:BUILD.hermit-abi-0.1.19.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__indexmap__1_9_1",
        url = "https://crates.io/api/v1/crates/indexmap/1.9.1/download",
        type = "tar.gz",
        sha256 = "10a35a97730320ffe8e2d410b5d3b69279b98d2c14bdb8b70ea89ecf7888d41e",
        strip_prefix = "indexmap-1.9.1",
        build_file = Label("//third_party/rust/crates/remote:BUILD.indexmap-1.9.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__inventory__0_2_3",
        url = "https://crates.io/api/v1/crates/inventory/0.2.3/download",
        type = "tar.gz",
        sha256 = "84344c6e0b90a9e2b6f3f9abe5cc74402684e348df7b32adca28747e0cef091a",
        strip_prefix = "inventory-0.2.3",
        build_file = Label("//third_party/rust/crates/remote:BUILD.inventory-0.2.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__itoa__1_0_3",
        url = "https://crates.io/api/v1/crates/itoa/1.0.3/download",
        type = "tar.gz",
        sha256 = "6c8af84674fe1f223a982c933a0ee1086ac4d4052aa0fb8060c12c6ad838e754",
        strip_prefix = "itoa-1.0.3",
        build_file = Label("//third_party/rust/crates/remote:BUILD.itoa-1.0.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__json5__0_4_1",
        url = "https://crates.io/api/v1/crates/json5/0.4.1/download",
        type = "tar.gz",
        sha256 = "96b0db21af676c1ce64250b5f40f3ce2cf27e4e47cb91ed91eb6fe9350b430c1",
        strip_prefix = "json5-0.4.1",
        build_file = Label("//third_party/rust/crates/remote:BUILD.json5-0.4.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__libc__0_2_133",
        url = "https://crates.io/api/v1/crates/libc/0.2.133/download",
        type = "tar.gz",
        sha256 = "c0f80d65747a3e43d1596c7c5492d95d5edddaabd45a7fcdb02b95f644164966",
        strip_prefix = "libc-0.2.133",
        build_file = Label("//third_party/rust/crates/remote:BUILD.libc-0.2.133.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__linked_hash_map__0_5_6",
        url = "https://crates.io/api/v1/crates/linked-hash-map/0.5.6/download",
        type = "tar.gz",
        sha256 = "0717cef1bc8b636c6e1c1bbdefc09e6322da8a9321966e8928ef80d20f7f770f",
        strip_prefix = "linked-hash-map-0.5.6",
        build_file = Label("//third_party/rust/crates/remote:BUILD.linked-hash-map-0.5.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__memchr__2_5_0",
        url = "https://crates.io/api/v1/crates/memchr/2.5.0/download",
        type = "tar.gz",
        sha256 = "2dffe52ecf27772e601905b7522cb4ef790d2cc203488bbd0e2fe85fcb74566d",
        strip_prefix = "memchr-2.5.0",
        build_file = Label("//third_party/rust/crates/remote:BUILD.memchr-2.5.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__num_traits__0_2_15",
        url = "https://crates.io/api/v1/crates/num-traits/0.2.15/download",
        type = "tar.gz",
        sha256 = "578ede34cf02f8924ab9447f50c28075b4d3e5b269972345e7e0372b38c6cdcd",
        strip_prefix = "num-traits-0.2.15",
        build_file = Label("//third_party/rust/crates/remote:BUILD.num-traits-0.2.15.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__once_cell__1_14_0",
        url = "https://crates.io/api/v1/crates/once_cell/1.14.0/download",
        type = "tar.gz",
        sha256 = "2f7254b99e31cad77da24b08ebf628882739a608578bb1bcdfc1f9c21260d7c0",
        strip_prefix = "once_cell-1.14.0",
        build_file = Label("//third_party/rust/crates/remote:BUILD.once_cell-1.14.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__os_str_bytes__6_3_0",
        url = "https://crates.io/api/v1/crates/os_str_bytes/6.3.0/download",
        type = "tar.gz",
        sha256 = "9ff7415e9ae3fff1225851df9e0d9e4e5479f947619774677a63572e55e80eff",
        strip_prefix = "os_str_bytes-6.3.0",
        build_file = Label("//third_party/rust/crates/remote:BUILD.os_str_bytes-6.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pest__2_3_1",
        url = "https://crates.io/api/v1/crates/pest/2.3.1/download",
        type = "tar.gz",
        sha256 = "cb779fcf4bb850fbbb0edc96ff6cf34fd90c4b1a112ce042653280d9a7364048",
        strip_prefix = "pest-2.3.1",
        build_file = Label("//third_party/rust/crates/remote:BUILD.pest-2.3.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pest_derive__2_3_1",
        url = "https://crates.io/api/v1/crates/pest_derive/2.3.1/download",
        type = "tar.gz",
        sha256 = "502b62a6d0245378b04ffe0a7fb4f4419a4815fce813bd8a0ec89a56e07d67b1",
        strip_prefix = "pest_derive-2.3.1",
        build_file = Label("//third_party/rust/crates/remote:BUILD.pest_derive-2.3.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pest_generator__2_3_1",
        url = "https://crates.io/api/v1/crates/pest_generator/2.3.1/download",
        type = "tar.gz",
        sha256 = "451e629bf49b750254da26132f1a5a9d11fd8a95a3df51d15c4abd1ba154cb6c",
        strip_prefix = "pest_generator-2.3.1",
        build_file = Label("//third_party/rust/crates/remote:BUILD.pest_generator-2.3.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pest_meta__2_3_1",
        url = "https://crates.io/api/v1/crates/pest_meta/2.3.1/download",
        type = "tar.gz",
        sha256 = "bcec162c71c45e269dfc3fc2916eaeb97feab22993a21bcce4721d08cd7801a6",
        strip_prefix = "pest_meta-2.3.1",
        build_file = Label("//third_party/rust/crates/remote:BUILD.pest_meta-2.3.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__proc_macro_error__1_0_4",
        url = "https://crates.io/api/v1/crates/proc-macro-error/1.0.4/download",
        type = "tar.gz",
        sha256 = "da25490ff9892aab3fcf7c36f08cfb902dd3e71ca0f9f9517bea02a73a5ce38c",
        strip_prefix = "proc-macro-error-1.0.4",
        build_file = Label("//third_party/rust/crates/remote:BUILD.proc-macro-error-1.0.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__proc_macro_error_attr__1_0_4",
        url = "https://crates.io/api/v1/crates/proc-macro-error-attr/1.0.4/download",
        type = "tar.gz",
        sha256 = "a1be40180e52ecc98ad80b184934baf3d0d29f979574e439af5a55274b35f869",
        strip_prefix = "proc-macro-error-attr-1.0.4",
        build_file = Label("//third_party/rust/crates/remote:BUILD.proc-macro-error-attr-1.0.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__proc_macro2__1_0_43",
        url = "https://crates.io/api/v1/crates/proc-macro2/1.0.43/download",
        type = "tar.gz",
        sha256 = "0a2ca2c61bc9f3d74d2886294ab7b9853abd9c1ad903a3ac7815c58989bb7bab",
        strip_prefix = "proc-macro2-1.0.43",
        build_file = Label("//third_party/rust/crates/remote:BUILD.proc-macro2-1.0.43.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__quote__1_0_21",
        url = "https://crates.io/api/v1/crates/quote/1.0.21/download",
        type = "tar.gz",
        sha256 = "bbe448f377a7d6961e30f5955f9b8d106c3f5e449d493ee1b125c1d43c2b5179",
        strip_prefix = "quote-1.0.21",
        build_file = Label("//third_party/rust/crates/remote:BUILD.quote-1.0.21.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__regex__1_6_0",
        url = "https://crates.io/api/v1/crates/regex/1.6.0/download",
        type = "tar.gz",
        sha256 = "4c4eb3267174b8c6c2f654116623910a0fef09c4753f8dd83db29c48a0df988b",
        strip_prefix = "regex-1.6.0",
        build_file = Label("//third_party/rust/crates/remote:BUILD.regex-1.6.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__regex_syntax__0_6_27",
        url = "https://crates.io/api/v1/crates/regex-syntax/0.6.27/download",
        type = "tar.gz",
        sha256 = "a3f87b73ce11b1619a3c6332f45341e0047173771e8b8b73f87bfeefb7b56244",
        strip_prefix = "regex-syntax-0.6.27",
        build_file = Label("//third_party/rust/crates/remote:BUILD.regex-syntax-0.6.27.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ryu__1_0_11",
        url = "https://crates.io/api/v1/crates/ryu/1.0.11/download",
        type = "tar.gz",
        sha256 = "4501abdff3ae82a1c1b477a17252eb69cee9e66eb915c1abaa4f44d873df9f09",
        strip_prefix = "ryu-1.0.11",
        build_file = Label("//third_party/rust/crates/remote:BUILD.ryu-1.0.11.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde__1_0_144",
        url = "https://crates.io/api/v1/crates/serde/1.0.144/download",
        type = "tar.gz",
        sha256 = "0f747710de3dcd43b88c9168773254e809d8ddbdf9653b84e2554ab219f17860",
        strip_prefix = "serde-1.0.144",
        build_file = Label("//third_party/rust/crates/remote:BUILD.serde-1.0.144.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_bytes__0_11_7",
        url = "https://crates.io/api/v1/crates/serde_bytes/0.11.7/download",
        type = "tar.gz",
        sha256 = "cfc50e8183eeeb6178dcb167ae34a8051d63535023ae38b5d8d12beae193d37b",
        strip_prefix = "serde_bytes-0.11.7",
        build_file = Label("//third_party/rust/crates/remote:BUILD.serde_bytes-0.11.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_derive__1_0_144",
        url = "https://crates.io/api/v1/crates/serde_derive/1.0.144/download",
        type = "tar.gz",
        sha256 = "94ed3a816fb1d101812f83e789f888322c34e291f894f19590dc310963e87a00",
        strip_prefix = "serde_derive-1.0.144",
        build_file = Label("//third_party/rust/crates/remote:BUILD.serde_derive-1.0.144.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_json__1_0_85",
        url = "https://crates.io/api/v1/crates/serde_json/1.0.85/download",
        type = "tar.gz",
        sha256 = "e55a28e3aaef9d5ce0506d0a14dbba8054ddc7e499ef522dd8b26859ec9d4a44",
        strip_prefix = "serde_json-1.0.85",
        build_file = Label("//third_party/rust/crates/remote:BUILD.serde_json-1.0.85.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_yaml__0_8_26",
        url = "https://crates.io/api/v1/crates/serde_yaml/0.8.26/download",
        type = "tar.gz",
        sha256 = "578a7433b776b56a35785ed5ce9a7e777ac0598aac5a6dd1b4b18a307c7fc71b",
        strip_prefix = "serde_yaml-0.8.26",
        build_file = Label("//third_party/rust/crates/remote:BUILD.serde_yaml-0.8.26.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__sha1__0_10_5",
        url = "https://crates.io/api/v1/crates/sha1/0.10.5/download",
        type = "tar.gz",
        sha256 = "f04293dc80c3993519f2d7f6f511707ee7094fe0c6d3406feb330cdb3540eba3",
        strip_prefix = "sha1-0.10.5",
        build_file = Label("//third_party/rust/crates/remote:BUILD.sha1-0.10.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__strsim__0_10_0",
        url = "https://crates.io/api/v1/crates/strsim/0.10.0/download",
        type = "tar.gz",
        sha256 = "73473c0e59e6d5812c5dfe2a064a6444949f089e20eec9a2e5506596494e4623",
        strip_prefix = "strsim-0.10.0",
        build_file = Label("//third_party/rust/crates/remote:BUILD.strsim-0.10.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__syn__1_0_100",
        url = "https://crates.io/api/v1/crates/syn/1.0.100/download",
        type = "tar.gz",
        sha256 = "52205623b1b0f064a4e71182c3b18ae902267282930c6d5462c91b859668426e",
        strip_prefix = "syn-1.0.100",
        build_file = Label("//third_party/rust/crates/remote:BUILD.syn-1.0.100.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__termcolor__1_1_3",
        url = "https://crates.io/api/v1/crates/termcolor/1.1.3/download",
        type = "tar.gz",
        sha256 = "bab24d30b911b2376f3a13cc2cd443142f0c81dda04c118693e35b3835757755",
        strip_prefix = "termcolor-1.1.3",
        build_file = Label("//third_party/rust/crates/remote:BUILD.termcolor-1.1.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__textwrap__0_15_1",
        url = "https://crates.io/api/v1/crates/textwrap/0.15.1/download",
        type = "tar.gz",
        sha256 = "949517c0cf1bf4ee812e2e07e08ab448e3ae0d23472aee8a06c985f0c8815b16",
        strip_prefix = "textwrap-0.15.1",
        build_file = Label("//third_party/rust/crates/remote:BUILD.textwrap-0.15.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__thiserror__1_0_35",
        url = "https://crates.io/api/v1/crates/thiserror/1.0.35/download",
        type = "tar.gz",
        sha256 = "c53f98874615aea268107765aa1ed8f6116782501d18e53d08b471733bea6c85",
        strip_prefix = "thiserror-1.0.35",
        build_file = Label("//third_party/rust/crates/remote:BUILD.thiserror-1.0.35.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__thiserror_impl__1_0_35",
        url = "https://crates.io/api/v1/crates/thiserror-impl/1.0.35/download",
        type = "tar.gz",
        sha256 = "f8b463991b4eab2d801e724172285ec4195c650e8ec79b149e6c2a8e6dd3f783",
        strip_prefix = "thiserror-impl-1.0.35",
        build_file = Label("//third_party/rust/crates/remote:BUILD.thiserror-impl-1.0.35.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__typenum__1_15_0",
        url = "https://crates.io/api/v1/crates/typenum/1.15.0/download",
        type = "tar.gz",
        sha256 = "dcf81ac59edc17cc8697ff311e8f5ef2d99fcbd9817b34cec66f90b6c3dfd987",
        strip_prefix = "typenum-1.15.0",
        build_file = Label("//third_party/rust/crates/remote:BUILD.typenum-1.15.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ucd_trie__0_1_5",
        url = "https://crates.io/api/v1/crates/ucd-trie/0.1.5/download",
        type = "tar.gz",
        sha256 = "9e79c4d996edb816c91e4308506774452e55e95c3c9de07b6729e17e15a5ef81",
        strip_prefix = "ucd-trie-0.1.5",
        build_file = Label("//third_party/rust/crates/remote:BUILD.ucd-trie-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicode_ident__1_0_4",
        url = "https://crates.io/api/v1/crates/unicode-ident/1.0.4/download",
        type = "tar.gz",
        sha256 = "dcc811dc4066ac62f84f11307873c4850cb653bfa9b1719cee2bd2204a4bc5dd",
        strip_prefix = "unicode-ident-1.0.4",
        build_file = Label("//third_party/rust/crates/remote:BUILD.unicode-ident-1.0.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__version_check__0_9_4",
        url = "https://crates.io/api/v1/crates/version_check/0.9.4/download",
        type = "tar.gz",
        sha256 = "49874b5167b65d7193b8aba1567f5c7d93d001cafc34600cee003eda787e483f",
        strip_prefix = "version_check-0.9.4",
        build_file = Label("//third_party/rust/crates/remote:BUILD.version_check-0.9.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi__0_3_9",
        url = "https://crates.io/api/v1/crates/winapi/0.3.9/download",
        type = "tar.gz",
        sha256 = "5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419",
        strip_prefix = "winapi-0.3.9",
        build_file = Label("//third_party/rust/crates/remote:BUILD.winapi-0.3.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_i686_pc_windows_gnu__0_4_0",
        url = "https://crates.io/api/v1/crates/winapi-i686-pc-windows-gnu/0.4.0/download",
        type = "tar.gz",
        sha256 = "ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6",
        strip_prefix = "winapi-i686-pc-windows-gnu-0.4.0",
        build_file = Label("//third_party/rust/crates/remote:BUILD.winapi-i686-pc-windows-gnu-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_util__0_1_5",
        url = "https://crates.io/api/v1/crates/winapi-util/0.1.5/download",
        type = "tar.gz",
        sha256 = "70ec6ce85bb158151cae5e5c87f95a8e97d2c0c4b001223f33a334e3ce5de178",
        strip_prefix = "winapi-util-0.1.5",
        build_file = Label("//third_party/rust/crates/remote:BUILD.winapi-util-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_x86_64_pc_windows_gnu__0_4_0",
        url = "https://crates.io/api/v1/crates/winapi-x86_64-pc-windows-gnu/0.4.0/download",
        type = "tar.gz",
        sha256 = "712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f",
        strip_prefix = "winapi-x86_64-pc-windows-gnu-0.4.0",
        build_file = Label("//third_party/rust/crates/remote:BUILD.winapi-x86_64-pc-windows-gnu-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__yaml_rust__0_4_5",
        url = "https://crates.io/api/v1/crates/yaml-rust/0.4.5/download",
        type = "tar.gz",
        sha256 = "56c1936c4cc7a1c9ab21a1ebb602eb942ba868cbd44a99cb7cdc5892335e1c85",
        strip_prefix = "yaml-rust-0.4.5",
        build_file = Label("//third_party/rust/crates/remote:BUILD.yaml-rust-0.4.5.bazel"),
    )
