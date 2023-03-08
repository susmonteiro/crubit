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

TEST_F(LifetimeAnalysisTest, ReturnArgumentPtr) {
  EXPECT_THAT(GetLifetimes(R"(
    int* target(int* a, int* b) {
      *a = *a + *b;
      return a;
    }
  )"),
              LifetimesAre({{"target", "a, b -> a"}}));
}

} // namespace
} // namespace lifetimes
} // namespace tidy
} // namespace clang
