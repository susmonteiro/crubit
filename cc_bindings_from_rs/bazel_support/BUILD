"""Disclaimer: This project is experimental, under heavy development, and should not
be used yet."""

load("@bazel_skylib//:bzl_library.bzl", "bzl_library")

package(default_applicable_licenses = ["//third_party/crubit:license"])

licenses(["notice"])

bzl_library(
    name = "cc_bindings_from_rust_rule_bzl",
    srcs = ["cc_bindings_from_rust_rule.bzl"],
    visibility = [
        "//visibility:private",  # Only private by automation, not intent. Owner may accept CLs adding visibility. See <internal link>.
    ],
)

alias(
    name = "cc_deps_for_bindings",
    actual = "//support/rs_std:rs_char",
    visibility = ["//visibility:public"],
)
