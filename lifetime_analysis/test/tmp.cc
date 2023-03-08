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

TEST_F(LifetimeAnalysisTest, TwoFunctions) {
  EXPECT_THAT(GetLifetimes(R"(
    int* target(int* a) {
      a = a + 1;
      return a;
    }

    int* mainTarget(int* b) {
      int* z = target(b);
      return z;
    }
  )"),
              LifetimesAre({{"mainTarget", "a -> a"}, {"target", "a -> a"}}));
}

} // namespace
} // namespace lifetimes
} // namespace tidy
} // namespace clang
