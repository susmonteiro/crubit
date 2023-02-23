// Part of the Crubit project, under the Apache License v2.0 with LLVM
// Exceptions. See /LICENSE for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

#ifndef DEVTOOLS_RUST_CC_INTEROP_LIFETIME_ANALYSIS_LIFETIME_ANALYSIS_H_
#define DEVTOOLS_RUST_CC_INTEROP_LIFETIME_ANALYSIS_LIFETIME_ANALYSIS_H_

#include <functional>
#include <string>
#include <variant>

#include "lifetime_analysis/lifetime_lattice.h"
#include "lifetime_analysis/object_repository.h"
#include "lifetime_analysis/points_to_map.h"
#include "lifetime_annotations/function_lifetimes.h"
#include "lifetime_annotations/lifetime.h"
#include "clang/AST/Decl.h"
#include "clang/AST/Expr.h"
#include "clang/AST/ExprCXX.h"
#include "clang/AST/Type.h"
#include "clang/Analysis/CFG.h"
#include "llvm/Support/Error.h"
#include "llvm/Support/raw_ostream.h"

namespace clang {
namespace tidy {
namespace lifetimes {

enum class TargetPointeeBehavior {
  kIgnore,
  kKeep,
};

// Updates constraints and points_to_map for an initialization of `dest` with
// `init_expr`. If `pointee_behavior` is kIgnore, existing pointees of `dest`
// will be ignored (this should be almost always the case, except when i.e.
// initializing field variables after the fact for class constructors).
void TransferInitializer(const Object *dest, clang::QualType type,
                         const ObjectRepository &object_repository,
                         const clang::Expr *init_expr,
                         TargetPointeeBehavior pointee_behavior,
                         PointsToMap &points_to_map,
                         LifetimeConstraints &constraints);

// Updates constraints and points_to_map whenever new pointees are added to the
// pointees of a given pointer.
void HandlePointsToSetExtension(const ObjectSet &pointers,
                                const ObjectSet &new_pointees,
                                clang::QualType pointer_type,
                                const ObjectRepository &object_repository,
                                PointsToMap &points_to_map,
                                LifetimeConstraints &constraints);

// Function to call to report a diagnostic.
// This has the same interface as ClangTidyCheck::diag().
using DiagnosticReporter = std::function<clang::DiagnosticBuilder(
    clang::SourceLocation, clang::StringRef, clang::DiagnosticIDs::Level)>;

class LifetimeAnalysis
    : public clang::dataflow::DataflowAnalysis<LifetimeAnalysis,
                                               LifetimeLattice> {
public:
  explicit LifetimeAnalysis(
      const clang::FunctionDecl *func, ObjectRepository &object_repository,
      const llvm::DenseMap<const clang::FunctionDecl *,
                           FunctionLifetimesOrError> &callee_lifetimes,
      const DiagnosticReporter &diag_reporter)
      : clang::dataflow::DataflowAnalysis<LifetimeAnalysis, LifetimeLattice>(
            func->getASTContext(), /*ApplyBuiltinTransfer=*/false),
        func_(func), object_repository_(object_repository),
        callee_lifetimes_(callee_lifetimes), diag_reporter_(diag_reporter) {}

  LifetimeLattice initialElement();

  std::string ToString(const LifetimeLattice &state);

  bool IsEqual(const LifetimeLattice &state1, const LifetimeLattice &state2);

  void transfer(const clang::CFGElement &elt, LifetimeLattice &state,
                clang::dataflow::Environment &environment);

  // TODO(yitzhakm): remove once https://reviews.llvm.org/D143920 is committed
  // and integrated downstream.
  void transfer(const clang::CFGElement *elt, LifetimeLattice &lattice,
                clang::dataflow::Environment &env) {
    transfer(*elt, lattice, env);
  }

private:
  const clang::FunctionDecl *func_;
  ObjectRepository &object_repository_;
  const llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
      &callee_lifetimes_;
  const DiagnosticReporter &diag_reporter_;
};

} // namespace lifetimes
} // namespace tidy
} // namespace clang

#endif // DEVTOOLS_RUST_CC_INTEROP_LIFETIME_ANALYSIS_LIFETIME_ANALYSIS_H_
