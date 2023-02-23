// Part of the Crubit project, under the Apache License v2.0 with LLVM
// Exceptions. See /LICENSE for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

// Tests for basic functionality.

#include "lifetime_analysis/test/lifetime_analysis_test.h"
#include "gmock/gmock.h"
#include "gtest/gtest.h"

namespace clang {
namespace tidy {
namespace lifetimes {
namespace {

TEST_F(LifetimeAnalysisTest, CompilationError) {
  // Check that we don't analyze code that doesn't compile.
  // This is a regression test -- we actually used to produce the lifetimes
  // "a -> a" for this test.
  EXPECT_THAT(GetLifetimes(R"(
    int* target(int* a) {
      undefined(&a);
      return a;
    }
  )"),
              LifetimesAre({{"", "Compilation error -- see log for details"}}));
}

} // namespace
} // namespace lifetimes
} // namespace tidy
} // namespace clang
