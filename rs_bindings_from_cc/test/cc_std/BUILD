load("@rules_rust//rust:defs.bzl", "rust_test")

package(default_applicable_licenses = ["//third_party/crubit:license"])

licenses(["notice"])

rust_test(
    name = "main",
    srcs = ["test.rs"],
    cc_deps = [
        "//rs_bindings_from_cc:cc_std",
    ],
    deps = ["//support:ctor"],
)

rust_test(
    name = "check_cc_std_in_deps",
    srcs = ["test.rs"],
    cc_deps = [
        "//rs_bindings_from_cc:cc_std",
    ],
    deps = ["//support:ctor"],
)

cc_library(
    name = "stl_in_deps",
    hdrs = ["empty.h"],
    deps = ["//third_party/stl"],
)

rust_test(
    name = "check_stl_in_deps",
    srcs = ["empty.rs"],
    cc_deps = [
        ":stl_in_deps",
        "//third_party/stl",
    ],
)
