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

// writing "DISABLED_" in the beginning of the name of the test disables it

TEST_F(LifetimeAnalysisTest, TwoFunctions) {
  EXPECT_THAT(GetLifetimes(R"(
    int* fn(int* x, int* y, int* z) {
      x = z;
      x = y;
      y = z;
      return x;
    }

  )"),
              LifetimesAre({{"fn", "a, b, c -> b"}}));
}

} // namespace
} // namespace lifetimes
} // namespace tidy
} // namespace clang
