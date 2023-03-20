// Part of the Crubit project, under the Apache License v2.0 with LLVM
// Exceptions. See /LICENSE for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

#include "lifetime_annotations/lifetime_annotations.h"

#include <iostream>
#include <string>
#include <utility>

#include "absl/status/status.h"
#include "common/status_test_matchers.h"
#include "lifetime_annotations/test/named_func_lifetimes.h"
#include "lifetime_annotations/test/run_on_code.h"
#include "lifetime_annotations/type_lifetimes.h"
#include "clang/ASTMatchers/ASTMatchFinder.h"
#include "clang/ASTMatchers/ASTMatchers.h"
#include "clang/Tooling/Tooling.h"
#include "llvm/Support/FormatVariadic.h"
#include "gmock/gmock.h"
#include "gtest/gtest.h"

// This file contains tests both for the "legacy" lifetime annotations
// (`[[clang::annotate("lifetimes", ...)]]` placed on a function declaration)
// and the newer annotations (`[[clang::annotate_type("lifetime", ...")]]`
// placed on a type). This is because we expect we may continue to use the
// "legacy" style of annotations in sidecar files.
//
// Some tests only test one style of annotation where testing the other style
// does not make sense for the particular test.

namespace clang {
namespace tidy {
namespace lifetimes {
namespace {

using crubit::IsOkAndHolds;
using crubit::StatusIs;
using testing::StartsWith;

bool IsOverloaded(const clang::FunctionDecl *func) {
  return !func->getDeclContext()->lookup(func->getDeclName()).isSingleResult();
}

std::string QualifiedName(const clang::FunctionDecl *func) {
  std::string str;
  llvm::raw_string_ostream ostream(str);
  func->printQualifiedName(ostream);
  if (IsOverloaded(func)) {
    ostream << "[" << StripAttributes(func->getType()).getAsString() << "]";
  }
  ostream.flush();
  return str;
}

// Prepends definitions for lifetime annotation macros to the code.
std::string WithLifetimeMacros(absl::string_view code) {
  std::string result = R"(
    // TODO(mboehme): We would prefer `$(...)` to be a variadic macro that
    // stringizes each of its macro arguments individually. This is possible but
    // requires some contortions: https://stackoverflow.com/a/5958315
    #define $(l) [[clang::annotate_type("lifetime", #l)]]
    #define $2(l1, l2) [[clang::annotate_type("lifetime", #l1, #l2)]]
    #define $3(l1, l2, l3) [[clang::annotate_type("lifetime", #l1, #l2, #l3)]]
  )";

  char buffer[128];

  for (char l = 'a'; l <= 'z'; ++l) {
    std::sprintf(buffer, "#define $%c $(%c)\n", l, l);
    absl::StrAppend(&result, buffer);
  }
  absl::StrAppend(&result, "#define $static $(static)\n");
  absl::StrAppend(&result, code);
  //   std::cout << "Resulting code" << result << std::endl;
  return result;
}

class LifetimeAnnotationsTest : public testing::Test {
protected:
  absl::StatusOr<NamedFuncLifetimes> GetNamedLifetimeAnnotations(
      absl::string_view code,
      const clang::tooling::FileContentMappings &file_contents =
          clang::tooling::FileContentMappings(),
      bool skip_templates = true) {
    absl::StatusOr<NamedFuncLifetimes> result;
    bool success = runOnCodeWithLifetimeHandlers(
        llvm::StringRef(code.data(), code.size()),
        [&result,
         skip_templates](clang::ASTContext &ast_context,
                         const LifetimeAnnotationContext &lifetime_context) {
          using clang::ast_matchers::findAll;
          using clang::ast_matchers::functionDecl;
          using clang::ast_matchers::match;

          NamedFuncLifetimes named_func_lifetimes;
          for (const auto &node :
               match(findAll(functionDecl().bind("func")), ast_context)) {
            if (const auto *func =
                    node.getNodeAs<clang::FunctionDecl>("func")) {
              // Skip various categories of function, unless explicitly
              // requested:
              // - Template instantiation don't contain any annotations that
              //   aren't present in the template itself, but they may contain
              //   reference-like types (which will obviously be unannotated),
              //   which will generate nuisance "lifetime elision not enabled"
              //   errors.
              // - Implicitly defaulted functions obviously cannot contain
              //   lifetime annotations. They will need to be handled through
              //   `AnalyzeDefaultedFunction()` in analyze.cc.
              if ((func->isTemplateInstantiation() && skip_templates) ||
                  (func->isDefaulted() && !func->isExplicitlyDefaulted())) {
                continue;
              }

              LifetimeSymbolTable symbol_table;
              llvm::Expected<FunctionLifetimes> func_lifetimes =
                  GetLifetimeAnnotations(func, lifetime_context, &symbol_table);

              std::string new_entry;
              if (func_lifetimes) {
                new_entry = NameLifetimes(*func_lifetimes, symbol_table);
              } else {
                new_entry = absl::StrCat(
                    "ERROR: ", llvm::toString(func_lifetimes.takeError()));
              }

              std::string func_name = QualifiedName(func);
              std::optional<llvm::StringRef> old_entry =
                  named_func_lifetimes.Get(func_name);
              if (old_entry.has_value()) {
                if (new_entry != old_entry.value()) {
                  result = absl::UnknownError(
                      llvm::formatv(
                          "Unexpectedly different lifetimes for function '{0}'."
                          "Old: '{1}'. New: '{2}'.",
                          func_name, old_entry.value(), new_entry)
                          .str());
                  return;
                }
              } else {
                named_func_lifetimes.Add(std::move(func_name),
                                         std::move(new_entry));
              }
            }
          }

          result = std::move(named_func_lifetimes);
        },
        {}, file_contents);

    if (!success) {
      return absl::UnknownError(
          "Error extracting lifetimes. (Compilation error?)");
    }

    return result;
  }
};

TEST_F(LifetimeAnnotationsTest, LifetimeAnnotation_Simple) {
  EXPECT_THAT(GetNamedLifetimeAnnotations(WithLifetimeMacros(R"(
        int* $a f(int* $a);
  )")),
              IsOkAndHolds(LifetimesAre({{"f", "a -> a"}})));
}

} // namespace
} // namespace lifetimes
} // namespace tidy
} // namespace clang
