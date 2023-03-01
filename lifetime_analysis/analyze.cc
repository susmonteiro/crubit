// Part of the Crubit project, under the Apache License v2.0 with LLVM
// Exceptions. See /LICENSE for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception

#include "lifetime_analysis/analyze.h"

#include <algorithm>
#include <iostream>
#include <map>
#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <variant>
#include <vector>

#include "absl/strings/str_cat.h"
#include "absl/strings/str_format.h"
#include "absl/strings/str_join.h"
#include "absl/strings/str_replace.h"
#include "lifetime_analysis/lifetime_analysis.h"
#include "lifetime_analysis/lifetime_lattice.h"
#include "lifetime_analysis/object_repository.h"
#include "lifetime_analysis/template_placeholder_support.h"
#include "lifetime_annotations/function_lifetimes.h"
#include "lifetime_annotations/lifetime.h"
#include "lifetime_annotations/lifetime_substitutions.h"
#include "lifetime_annotations/pointee_type.h"
#include "lifetime_annotations/type_lifetimes.h"
#include "clang/AST/Decl.h"
#include "clang/AST/DeclCXX.h"
#include "clang/AST/DeclTemplate.h"
#include "clang/AST/Expr.h"
#include "clang/AST/ExprCXX.h"
#include "clang/AST/Type.h"
#include "clang/ASTMatchers/ASTMatchFinder.h"
#include "clang/ASTMatchers/ASTMatchers.h"
#include "clang/Analysis/CFG.h"
#include "clang/Analysis/FlowSensitive/ControlFlowContext.h"
#include "clang/Analysis/FlowSensitive/DataflowAnalysis.h"
#include "clang/Analysis/FlowSensitive/DataflowEnvironment.h"
#include "clang/Analysis/FlowSensitive/WatchedLiteralsSolver.h"
#include "clang/Index/USRGeneration.h"
#include "clang/Lex/Lexer.h"
#include "llvm/ADT/ArrayRef.h"
#include "llvm/ADT/SmallPtrSet.h"
#include "llvm/ADT/StringRef.h"
#include "llvm/Support/Error.h"

namespace clang {
namespace tidy {
namespace lifetimes {
namespace {

struct VisitedCallStackEntry {
  const clang::FunctionDecl *func;
  bool in_cycle;
  bool in_overrides_traversal;
};

void debug(std::string text) {
  std::cout << "\033[1;33m[analyze.cc] >> \033[0m" << text
            << std::endl; // DEBUG
}

// A map from base methods to overriding methods.
using BaseToOverrides =
    llvm::DenseMap<const clang::CXXMethodDecl *,
                   llvm::SmallPtrSet<const clang::CXXMethodDecl *, 2>>;

// Enforce the invariant that an object of static lifetime should only point at
// other objects of static lifetime.
llvm::Error PropagateStaticToPointees(LifetimeSubstitutions &subst,
                                      const PointsToMap &points_to_map) {
  std::vector<const Object *> pointees =
      points_to_map.GetAllPointersWithLifetime(Lifetime::Static());

  llvm::DenseSet<const Object *> visited;

  while (!pointees.empty()) {
    const Object *cur = pointees.back();
    pointees.pop_back();
    visited.insert(cur);
    if (cur->GetLifetime().IsLocal()) {
      return llvm::createStringError(
          llvm::inconvertibleErrorCode(),
          "attempted to make a pointer of static lifetime point at an object "
          "of local lifetime");
    }
    if (cur->GetLifetime() != Lifetime::Static()) {
      subst.Add(cur->GetLifetime(), Lifetime::Static());
    }

    for (const Object *pointee : points_to_map.GetPointerPointsToSet(cur)) {
      if (!visited.count(pointee)) {
        pointees.push_back(pointee);
      }
    }
  }

  return llvm::Error::success();
}

// DO NOT use this function on untrusted input.
// TODO(veluca): ideally, this function should be replaced with one from a
// fuzzed library. However, as the way it is used doesn't have significant
// security implications (its input is trusted, coming from tests, and its
// output is unused except sometimes to produce a graphviz .dot file), and as
// the logic for HTML escaping is simple enough, this function is reasonable to
// use here.
std::string EscapeHtmlChars(absl::string_view input) {
  std::string escaped;
  escaped.reserve(input.size());
  for (auto c : input) {
    switch (c) {
    case '\'':
      escaped += "&#39;";
      break;
    case '"':
      escaped += "&quot;";
      break;
    case '<':
      escaped += "&lt;";
      break;
    case '>':
      escaped += "&gt;";
      break;
    case '&':
      escaped += "&amp;";
      break;
    default:
      escaped += c;
    }
  }
  return escaped;
}

std::string VariableLabel(absl::string_view name, const Object *object) {
  return absl::StrFormat("<<b>%s</b> (%s)>", EscapeHtmlChars(name),
                         EscapeHtmlChars(object->DebugString()));
}

std::string PointsToEdgesDot(const ObjectRepository &object_repository,
                             const PointsToMap &points_to_map,
                             absl::string_view name_prefix) {

  debug("PointsToEdgesDot"); // DEBUG

  std::vector<std::string> lines;
  llvm::DenseSet<const Object *> all_objects, var_objects;

  for (auto [pointer, points_to_set] : points_to_map.PointerPointsTos()) {
    all_objects.insert(pointer);
    for (auto points_to : points_to_set) {
      all_objects.insert(points_to);
      lines.push_back(absl::StrFormat(R"("%1$s%2$s" -> "%1$s%3$s")",
                                      name_prefix, pointer->DebugString(),
                                      points_to->DebugString()));
    }
  }

  for (auto [key, field_object] : object_repository.GetFieldObjects()) {
    auto [struct_object, field] = key;
    lines.push_back(absl::StrFormat(
        R"("%1$s%2$s" -> "%1$s%3$s" [style=dashed label="%4$s"])", name_prefix,
        struct_object->DebugString(), field_object->DebugString(),
        field->getNameAsString()));
  }

  for (auto [key, base_object] : object_repository.GetBaseObjects()) {
    auto [struct_object, base] = key;
    lines.push_back(absl::StrFormat(
        R"("%1$s%2$s" -> "%1$s%3$s" [style=dashed label="%4$s"])", name_prefix,
        struct_object->DebugString(), base_object->DebugString(),
        clang::QualType(base, 0).getAsString()));
  }

  if (object_repository.GetThisObject().has_value()) {
    var_objects.insert(*object_repository.GetThisObject());
    lines.push_back(absl::StrFormat(
        "\"%s%s\"[label=%s]", name_prefix,
        (*object_repository.GetThisObject())->DebugString(),
        VariableLabel("this", *object_repository.GetThisObject())));
  }

  for (auto [decl, object] : object_repository) {
    var_objects.insert(object);
    lines.push_back(absl::StrFormat(
        "\"%s%s\"[label=%s]", name_prefix, object->DebugString(),
        VariableLabel(decl->getNameAsString(), object)));
  }

  var_objects.insert(object_repository.GetReturnObject());
  lines.push_back(absl::StrFormat(
      "\"%s%s\"[label=%s]", name_prefix,
      object_repository.GetReturnObject()->DebugString(),
      VariableLabel("return", object_repository.GetReturnObject())));

  for (const Object *object : all_objects) {
    if (!var_objects.contains(object)) {
      lines.push_back(absl::StrFormat(R"("%1$s%2$s"[label="%2$s"])",
                                      name_prefix, object->DebugString()));
    }
  }

  for (auto [_, object] : object_repository.GetFieldObjects()) {
    if (!var_objects.contains(object)) {
      lines.push_back(absl::StrFormat(R"("%1$s%2$s"[label="%2$s"])",
                                      name_prefix, object->DebugString()));
    }
  }

  for (auto [_, object] : object_repository.GetBaseObjects()) {
    if (!var_objects.contains(object)) {
      lines.push_back(absl::StrFormat(R"("%1$s%2$s"[label="%2$s"])",
                                      name_prefix,
                                      VariableLabel("this", object)));
    }
  }

  lines.push_back("");

  return absl::StrJoin(lines, ";\n");
}

std::string PointsToGraphDot(const ObjectRepository &object_repository,
                             const PointsToMap &points_to_map) {
  return absl::StrCat("digraph d {\n",
                      PointsToEdgesDot(object_repository, points_to_map, ""),
                      "}");
}

std::string ConstraintsEdgesDot(const ObjectRepository &object_repository,
                                const LifetimeConstraints &constraints,
                                absl::string_view name_prefix) {
  std::vector<std::string> lines;

  llvm::DenseSet<Lifetime> all_lifetimes;
  for (const auto &cstr : constraints.AllConstraints()) {
    lines.push_back(absl::StrFormat(R"("%1$s%2$d" -> "%1$s%3$d")", name_prefix,
                                    cstr.second.Id(), cstr.first.Id()));
    all_lifetimes.insert(cstr.first);
    all_lifetimes.insert(cstr.second);
  }

  for (auto lftm : all_lifetimes) {
    lines.push_back(absl::StrFormat(R"("%s%d"[label="%s"])", name_prefix,
                                    lftm.Id(), lftm.DebugString()));
  }

  return absl::StrJoin(lines, ";\n");
}

std::string ConstraintsDot(const ObjectRepository &object_repository,
                           const LifetimeConstraints &constraints) {
  return absl::StrCat("digraph d {\n",
                      ConstraintsEdgesDot(object_repository, constraints, ""),
                      "}");
}

std::string CfgBlockLabel(const clang::CFGBlock *block, const clang::CFG &cfg,
                          const clang::ASTContext &ast_context) {

  debug("CfgBlockLabel");

  std::string block_name = absl::StrCat("B", block->getBlockID());
  if (block == &cfg.getEntry()) {
    absl::StrAppend(&block_name, " (ENTRY)");
  } else if (block == &cfg.getExit()) {
    absl::StrAppend(&block_name, " (EXIT)");
  }

  std::string label =
      absl::StrFormat("<tr><td>%s</td></tr>", EscapeHtmlChars(block_name));

  clang::SourceRange range;
  for (const auto &element : *block) {
    if (auto cfg_stmt = element.getAs<clang::CFGStmt>()) {
      clang::SourceRange stmt_range = cfg_stmt->getStmt()->getSourceRange();
      if (range.isInvalid()) {
        range = stmt_range;
      } else {
        if (stmt_range.getBegin() < range.getBegin()) {
          range.setBegin(stmt_range.getBegin());
        }
        if (stmt_range.getEnd() > range.getEnd()) {
          range.setEnd(stmt_range.getEnd());
        }
      }
    }
  }

  if (range.isValid()) {
    const clang::SourceManager &source_manager = ast_context.getSourceManager();
    clang::StringRef filename = source_manager.getFilename(range.getBegin());
    unsigned line_begin =
        source_manager.getSpellingLineNumber(range.getBegin());
    unsigned col_begin =
        source_manager.getSpellingColumnNumber(range.getBegin());
    unsigned line_end = source_manager.getSpellingLineNumber(range.getEnd());
    unsigned col_end = source_manager.getSpellingColumnNumber(range.getEnd());

    absl::StrAppendFormat(&label, "<tr><td>%s:%u:%u-%u:%u</td></tr>",
                          EscapeHtmlChars(filename.str()), line_begin,
                          col_begin, line_end, col_end);

    absl::StrAppendFormat(
        &label, "<tr><td>%s</td></tr>",
        EscapeHtmlChars(clang::Lexer::getSourceText(
                            clang::CharSourceRange::getTokenRange(range),
                            source_manager, ast_context.getLangOpts())
                            .str()));
  }

  return absl::StrFormat("<<table border=\"0\">%s</table>>", label);
}

std::string CreateCfgDot(
    const clang::CFG &cfg, const clang::ASTContext &ast_context,
    const std::vector<
        llvm::Optional<clang::dataflow::DataflowAnalysisState<LifetimeLattice>>>
        &block_to_output_state,
    const ObjectRepository &object_repository) {
  std::string result = "digraph d {\ncompound=true;\nedge [minlen=2];\n";

  for (const clang::CFGBlock *block : cfg) {
    unsigned id = block->getBlockID();

    absl::StrAppendFormat(&result, "subgraph cluster%u {\n", id);

    absl::StrAppendFormat(&result, "label=%s;\n",
                          CfgBlockLabel(block, cfg, ast_context));

    absl::StrAppend(&result, "{\nrank=source;\n");
    absl::StrAppendFormat(
        &result,
        "B%usource [style=\"invis\",width=0,height=0,fixedsize=true];\n", id);
    absl::StrAppend(&result, "}\n");
    absl::StrAppend(&result, "{\nrank=sink;\n");
    absl::StrAppendFormat(
        &result, "B%usink [style=\"invis\",width=0,height=0,fixedsize=true];\n",
        id);
    absl::StrAppend(&result, "}\n");

    const auto block_state = block_to_output_state.at(id);
    if (block_state) {
      auto lattice = block_state->Lattice;
      if (!lattice.IsError()) {
        absl::StrAppend(&result,
                        PointsToEdgesDot(object_repository, lattice.PointsTo(),
                                         absl::StrCat("B", id, "_")));
        absl::StrAppend(&result, ConstraintsEdgesDot(
                                     object_repository, lattice.Constraints(),
                                     absl::StrCat("B", id, "_cstr_")));
      }
    }

    absl::StrAppend(&result, "}\n");
  }

  for (const clang::CFGBlock *block : cfg) {
    for (const clang::CFGBlock *succ : block->succs()) {
      absl::StrAppendFormat(
          &result,
          "B%1$usink -> B%2$usource [ltail=cluster%1$u,lhead=cluster%2$u];\n",
          block->getBlockID(), succ->getBlockID());
    }
  }

  absl::StrAppend(&result, "}");

  return result;
}

// TODO(veluca): this really ought to happen in the dataflow framework/CFG, but
// at the moment only the *expressions* in initializers get added, not
// initialization itself.
void ExtendPointsToMapAndConstraintsWithInitializers(
    const clang::CXXConstructorDecl *constructor,
    const ObjectRepository &object_repository, PointsToMap &points_to_map,
    LifetimeConstraints &constraints) {
  auto this_object = object_repository.GetThisObject();
  if (!this_object.has_value()) {
    assert(false);
    return;
  }
  for (const auto *init : constructor->inits()) {
    if (!init->isAnyMemberInitializer())
      continue;
    const clang::FieldDecl *field = init->getMember();
    const auto *init_expr = init->getInit();
    if (clang::isa<clang::CXXDefaultInitExpr>(init_expr)) {
      init_expr = field->getInClassInitializer();
    }
    if (!IsInitExprInitializingARecordObject(init_expr)) {
      TransferInitializer(
          object_repository.GetFieldObject(this_object.value(), field),
          field->getType(), object_repository, init_expr,
          TargetPointeeBehavior::kKeep, points_to_map, constraints);
    }
  }
}

llvm::Error ConstrainLifetimes(FunctionLifetimes &base,
                               const FunctionLifetimes &constraining) {
  auto constraints =
      LifetimeConstraints::ForCallableSubstitution(base, constraining);
  return constraints.ApplyToFunctionLifetimes(base);
}

struct FunctionAnalysis {
  ObjectRepository object_repository;
  PointsToMap points_to_map;
  LifetimeConstraints constraints;
  LifetimeSubstitutions subst;
};

const CXXConstructorDecl *GetDefaultConstructor(const CXXRecordDecl *record) {
  for (const CXXConstructorDecl *ctor : record->ctors()) {
    if (ctor->isDefaultConstructor()) {
      return ctor;
    }
  }
  return nullptr;
}

llvm::Error TransferDefaultConstructor(
    const clang::CXXConstructorDecl *default_ctor, const Object *this_object,
    ObjectRepository &object_repository, PointsToMap &points_to_map,
    LifetimeConstraints &constraints, ObjectSet &single_valued_objects,
    const llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
        &callee_lifetimes) {
  assert(callee_lifetimes.count(default_ctor->getCanonicalDecl()));

  const FunctionLifetimesOrError &ctor_lifetimes_or_error =
      callee_lifetimes.lookup(default_ctor->getCanonicalDecl());
  if (!std::holds_alternative<FunctionLifetimes>(ctor_lifetimes_or_error)) {
    return llvm::createStringError(
        llvm::inconvertibleErrorCode(),
        absl::StrCat("No lifetimes for constructor ",
                     default_ctor->getNameAsString()));
  }
  const FunctionLifetimes &ctor_lifetimes =
      std::get<FunctionLifetimes>(ctor_lifetimes_or_error);

  // Similar to handling of constructor calls; however, this is simpler because
  // there is only the "this" argument (as this is the default constructor).
  // Moreover, since we don't run dataflow, we create the objects on the fly.
  clang::QualType this_type = default_ctor->getThisType();
  // "object" for the `this` pointer itself.
  const Object *placeholder_this_ptr_object =
      object_repository.CreateObjectsRecursively(
          ObjectLifetimes(Lifetime::CreateVariable(),
                          ctor_lifetimes.GetThisLifetimes()),
          points_to_map);
  HandlePointsToSetExtension({placeholder_this_ptr_object}, {this_object},
                             this_type, object_repository, points_to_map,
                             constraints);
  return llvm::Error::success();
}

llvm::Error AnalyzeDefaultedDefaultConstructor(
    const clang::CXXConstructorDecl *ctor,
    const llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
        &callee_lifetimes,
    ObjectRepository &object_repository, PointsToMap &points_to_map,
    LifetimeConstraints &constraints, ObjectSet &single_valued_objects) {
  assert(ctor->isDefaulted() && ctor->isDefaultConstructor());

  std::optional<const Object *> this_object_maybe =
      object_repository.GetThisObject();
  if (!this_object_maybe.has_value()) {
    llvm::report_fatal_error("didn't find `this` object for constructor");
  }
  const Object *this_object = *this_object_maybe;

  const clang::CXXRecordDecl *record = ctor->getParent();
  for (const CXXBaseSpecifier &base : record->bases()) {
    if (const clang::CXXRecordDecl *base_record =
            base.getType()->getAsCXXRecordDecl()) {
      if (const clang::CXXConstructorDecl *base_ctor =
              GetDefaultConstructor(base_record)) {
        const Object *base_this_object =
            object_repository.GetBaseClassObject(this_object, base.getType());
        if (llvm::Error err = TransferDefaultConstructor(
                base_ctor, base_this_object, object_repository, points_to_map,
                constraints, single_valued_objects, callee_lifetimes)) {
          return err;
        }
      }
    }
  }
  for (const clang::FieldDecl *field : record->fields()) {
    if (const clang::CXXRecordDecl *field_record =
            field->getType()->getAsCXXRecordDecl()) {
      if (const clang::CXXConstructorDecl *field_ctor =
              GetDefaultConstructor(field_record)) {
        const Object *field_this_object =
            object_repository.GetFieldObject(this_object, field);
        if (llvm::Error err = TransferDefaultConstructor(
                field_ctor, field_this_object, object_repository, points_to_map,
                constraints, single_valued_objects, callee_lifetimes)) {
          return err;
        }
      }
    }
  }

  return llvm::Error::success();
}

llvm::Error AnalyzeDefaultedFunction(
    const clang::FunctionDecl *func,
    const llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
        &callee_lifetimes,
    ObjectRepository &object_repository, PointsToMap &points_to_map,
    LifetimeConstraints &constraints, ObjectSet &single_valued_objects) {
  assert(func->isDefaulted());

  // TODO(b/230693710): Add complete support for defaulted functions.

  if (const auto *ctor = clang::dyn_cast<clang::CXXConstructorDecl>(func)) {
    if (ctor->isDefaultConstructor()) {
      return AnalyzeDefaultedDefaultConstructor(
          ctor, callee_lifetimes, object_repository, points_to_map, constraints,
          single_valued_objects);
    }
  }

  return llvm::createStringError(llvm::inconvertibleErrorCode(),
                                 "unsupported type of defaulted function");
}

llvm::Error AnalyzeFunctionBody(
    const clang::FunctionDecl *func,
    const llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
        &callee_lifetimes,
    const DiagnosticReporter &diag_reporter,
    ObjectRepository &object_repository, PointsToMap &points_to_map,
    LifetimeConstraints &constraints, std::string *cfg_dot) {
  auto cfctx = clang::dataflow::ControlFlowContext::build(
      func, *func->getBody(), func->getASTContext());
  if (!cfctx)
    return cfctx.takeError();

  clang::dataflow::DataflowAnalysisContext analysis_context(
      std::make_unique<clang::dataflow::WatchedLiteralsSolver>());
  clang::dataflow::Environment environment(analysis_context);

  LifetimeAnalysis analysis(func, object_repository, callee_lifetimes,
                            diag_reporter);

  llvm::Expected<std::vector<
      llvm::Optional<clang::dataflow::DataflowAnalysisState<LifetimeLattice>>>>
      maybe_block_to_output_state =
          clang::dataflow::runDataflowAnalysis(*cfctx, analysis, environment);
  if (!maybe_block_to_output_state) {
    return maybe_block_to_output_state.takeError();
  }
  auto &block_to_output_state = *maybe_block_to_output_state;

  const auto exit_block_state =
      block_to_output_state.at(cfctx->getCFG().getExit().getBlockID());
  if (!exit_block_state.has_value()) {
    assert(false);
    return llvm::createStringError(
        llvm::inconvertibleErrorCode(),
        absl::StrCat("CFG exit block for '", func->getNameAsString(),
                     "' unexpectedly does not exist"));
  }

  auto exit_lattice = exit_block_state->Lattice;
  if (exit_lattice.IsError()) {
    return llvm::createStringError(llvm::inconvertibleErrorCode(),
                                   exit_lattice.Error());
  }

  points_to_map = exit_lattice.PointsTo();
  constraints = exit_lattice.Constraints();

  // Adding initializers to the PointsToMap *before* dataflow analysis is
  // problematic because the expressions do not have a lifetime yet in the map
  // itself.
  // However, member access in a struct does not ever produce lifetimes that
  // depend on what those members are initialized to - lifetimes of members
  // (or things that members point to) are either the same as the lifetime of
  // this, or a lifetime parameter of the struct, so processing initializers
  // afterwards is correct.
  if (auto *constructor = clang::dyn_cast<clang::CXXConstructorDecl>(func)) {
    ExtendPointsToMapAndConstraintsWithInitializers(
        constructor, object_repository, points_to_map, constraints);
  }

  // Extend the constraint set with constraints of the form "'a >= 'static" for
  // every object that is (transitively) reachable from a 'static object.
  std::vector<const Object *> stack =
      points_to_map.GetAllPointersWithLifetime(Lifetime::Static());
  llvm::DenseSet<const Object *> visited;
  while (!stack.empty()) {
    const Object *obj = stack.back();
    stack.pop_back();
    if (visited.contains(obj)) {
      continue;
    }
    visited.insert(obj);
    constraints.AddOutlivesConstraint(Lifetime::Static(), obj->GetLifetime());
    for (auto pointee : points_to_map.GetPointerPointsToSet(obj)) {
      stack.push_back(pointee);
    }
  }

  if (cfg_dot) {
    *cfg_dot = CreateCfgDot(cfctx->getCFG(), func->getASTContext(),
                            block_to_output_state, object_repository);
  }

  return llvm::Error::success();
}

llvm::Expected<FunctionAnalysis> AnalyzeSingleFunction(
    const clang::FunctionDecl *func,
    const llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
        &callee_lifetimes,
    const DiagnosticReporter &diag_reporter, FunctionDebugInfoMap *debug_info) {
  FunctionAnalysis analysis{.object_repository = ObjectRepository(func)};

  const auto *cxxmethod = clang::dyn_cast<clang::CXXMethodDecl>(func);
  if (cxxmethod && cxxmethod->isPure()) {
    return analysis;
  }

  func = func->getDefinition();
  assert(func != nullptr);

  // Unconditionally use our custom logic to analyze defaulted functions, even
  // if they happen to have a body (because something caused Sema to create a
  // body for them). We don't want the code path for defaulted functions to
  // change based on whether a body happened to be created for them, and we
  // want to make sure we always exercise our logic for defaulted functions in
  // tests.
  // TODO(b/230693710): We currently only support analyzing defaulted default
  // constructors, so for other defaulted functions, we currently fall back to
  // AnalyzeFunctionBody() (if they do have a body).
  const auto *ctor = clang::dyn_cast<clang::CXXConstructorDecl>(func);
  bool can_analyze_defaulted_func =
      ctor != nullptr && ctor->isDefaultConstructor();
  if (func->isDefaulted() && can_analyze_defaulted_func) {
    // Single-valued objects are only used during the analysis itself, so no
    // need to keep track of them past this point.
    ObjectSet single_valued_objects =
        analysis.object_repository.InitialSingleValuedObjects();
    if (llvm::Error err = AnalyzeDefaultedFunction(
            func, callee_lifetimes, analysis.object_repository,
            analysis.points_to_map, analysis.constraints,
            single_valued_objects)) {
      return std::move(err);
    }
  } else if (func->getBody()) {
    std::string *cfg_dot = debug_info ? &(*debug_info)[func].cfg_dot : nullptr;
    if (llvm::Error err = AnalyzeFunctionBody(
            func, callee_lifetimes, diag_reporter, analysis.object_repository,
            analysis.points_to_map, analysis.constraints, cfg_dot)) {
      return std::move(err);
    }
  } else {
    return llvm::createStringError(llvm::inconvertibleErrorCode(),
                                   "Declaration-only!");
  }

  if (debug_info) {
    std::string ast;
    llvm::raw_string_ostream os(ast);
    func->dump(os);
    os.flush();
    (*debug_info)[func].ast = std::move(ast);
    (*debug_info)[func].object_repository =
        analysis.object_repository.DebugString();
    (*debug_info)[func].points_to_map_dot =
        PointsToGraphDot(analysis.object_repository, analysis.points_to_map);
    (*debug_info)[func].constraints_dot =
        ConstraintsDot(analysis.object_repository, analysis.constraints);
  }

  if (llvm::Error err =
          PropagateStaticToPointees(analysis.subst, analysis.points_to_map)) {
    return std::move(err);
  }

  return analysis;
}

llvm::Error DiagnoseReturnLocal(const clang::FunctionDecl *func,
                                const FunctionLifetimes &lifetimes,
                                const DiagnosticReporter &diag_reporter) {
  auto contains_local = [](const ValueLifetimes &lifetimes) {
    return lifetimes.HasAny(&Lifetime::IsLocal);
  };

  for (unsigned i = 0; i < func->getNumParams(); ++i) {
    const clang::ParmVarDecl *param = func->getParamDecl(i);
    if (contains_local(lifetimes.GetParamLifetimes(i))) {
      std::string error_msg = absl::StrFormat(
          "function returns reference to a local through parameter '%s'",
          param->getNameAsString());
      diag_reporter(param->getBeginLoc(), error_msg,
                    clang::DiagnosticIDs::Error);
      return llvm::createStringError(llvm::inconvertibleErrorCode(), error_msg);
    }
  }

  if (const auto *method = clang::dyn_cast<clang::CXXMethodDecl>(func);
      method && !method->isStatic() &&
      contains_local(lifetimes.GetThisLifetimes())) {
    std::string error_msg =
        "function returns reference to a local through 'this'";
    diag_reporter(func->getBeginLoc(), error_msg, clang::DiagnosticIDs::Error);
    return llvm::createStringError(llvm::inconvertibleErrorCode(), error_msg);
  }

  if (contains_local(lifetimes.GetReturnLifetimes())) {
    std::string error_msg = "function returns reference to a local";
    diag_reporter(func->getBeginLoc(), error_msg, clang::DiagnosticIDs::Error);
    return llvm::createStringError(llvm::inconvertibleErrorCode(), error_msg);
  }

  return llvm::Error::success();
}

// Step x: Given a points-to set, return the lifetime
// * get original function lifetimes
// ! find out about this
// * apply the constraints to the function lifetimes
// ! also see what this means
// * diagnose return local
// ! find out about this
// Constructs the FunctionLifetimes for a function, given a PointsToMap,
// ObjectRepository, and LifetimeSubstitutions that have been built from the
// function's body, which would include the function's parameters. It's also
// possible to call this function with an empty inputs in order to generate
// a FunctionLifetimes that matches the function's signature but without any
// constraints (i.e. each lifetime that appears would be independent).
llvm::Expected<FunctionLifetimes>
ConstructFunctionLifetimes(const clang::FunctionDecl *func,
                           FunctionAnalysis analysis,
                           const DiagnosticReporter &diag_reporter) {

  debug("ConstructFunctionLifetimes"); // DEBUG

  if (func->getDefinition()) {
    func = func->getDefinition();
  } else { // * Virtual method
    // This can happen only when `func` is a pure virtual method.
    const auto *cxxmethod = clang::dyn_cast<clang::CXXMethodDecl>(func);
    assert(cxxmethod && cxxmethod->isPure());
    // Pure virtual member functions can only ever have a single declaration,
    // so we know we're already looking at the canonical declaration.
    if (++cxxmethod->redecls_begin() != cxxmethod->redecls_end()) {
      assert(false);
      func = func->getCanonicalDecl();
    }
  }
  // * object repository:
  // + objects used in the lifetime analysis of a single function
  // * points_to_map
  // ? lifetime constraints
  // * lifetime substitutions: built from the function's body
  auto &[object_repository, points_to_map, constraints, subst] = analysis;

  FunctionLifetimes result;

  result = object_repository.GetOriginalFunctionLifetimes();
  if (llvm::Error err = constraints.ApplyToFunctionLifetimes(result)) {
    return std::move(err);
  }

  if (llvm::Error err = DiagnoseReturnLocal(func, result, diag_reporter)) {
    return std::move(err);
  }

  return result;
}

llvm::Expected<llvm::DenseSet<const clang::FunctionDecl *>>
GetDefaultedFunctionCallees(const clang::FunctionDecl *func) {
  assert(func->isDefaulted());

  debug("GetDefaultedFunctionCallees"); // DEBUG

  // TODO(b/230693710): Add complete support for defaulted functions.

  if (const auto *ctor = clang::dyn_cast<clang::CXXConstructorDecl>(func)) {
    if (ctor->isDefaultConstructor()) {
      llvm::DenseSet<const clang::FunctionDecl *> callees;
      const clang::CXXRecordDecl *record = ctor->getParent();
      for (const CXXBaseSpecifier &base : record->bases()) {
        if (const clang::CXXRecordDecl *base_record =
                base.getType()->getAsCXXRecordDecl()) {
          if (const clang::CXXConstructorDecl *base_ctor =
                  GetDefaultConstructor(base_record)) {
            callees.insert(base_ctor);
          }
        }
      }
      for (const clang::FieldDecl *field : record->fields()) {
        if (const clang::CXXRecordDecl *field_record =
                field->getType()->getAsCXXRecordDecl()) {
          if (const clang::CXXConstructorDecl *field_ctor =
                  GetDefaultConstructor(field_record)) {
            callees.insert(field_ctor);
          }
        }
      }
      return callees;
    }
  }

  return llvm::createStringError(llvm::inconvertibleErrorCode(),
                                 "unsupported type of defaulted function");
}

llvm::Expected<llvm::DenseSet<const clang::FunctionDecl *>>
GetCallees(const clang::FunctionDecl *func) {
  using clang::ast_matchers::anyOf;
  using clang::ast_matchers::cxxConstructExpr;
  using clang::ast_matchers::declRefExpr;
  using clang::ast_matchers::expr;
  using clang::ast_matchers::findAll;
  using clang::ast_matchers::functionDecl;
  using clang::ast_matchers::hasDeclaration;
  using clang::ast_matchers::match;
  using clang::ast_matchers::memberExpr;
  using clang::ast_matchers::to;

  debug("Inside GetCallees"); // DEBUG

  func = func->getDefinition();

  if (!func)
    return llvm::DenseSet<const clang::FunctionDecl *>();

  const clang::Stmt *body = func->getBody();
  if (!body) {
    // TODO(b/230693710): Do this unconditionally for defaulted functions, even
    // if they happen to have a body (because something caused Sema to create a
    // body for them). We can't do this yet because we don't have full support
    // for defaulted functions yet, so we would break tests where we happen to
    // have a body for the defaulted function today.
    if (func->isDefaulted()) {
      return GetDefaultedFunctionCallees(func);
    }

    return llvm::createStringError(llvm::inconvertibleErrorCode(),
                                   "Declaration-only!");
  }

  llvm::SmallVector<const clang::Stmt *> body_parts;

  body_parts.push_back(body);

  if (const auto *constructor =
          clang::dyn_cast<clang::CXXConstructorDecl>(func)) {
    for (const auto *init : constructor->inits()) {
      body_parts.push_back(init->getInit());
    }
  }

  llvm::DenseSet<const clang::FunctionDecl *> callees;
  for (const auto &body_part : body_parts) {
    for (const auto &node : match(
             findAll(expr(anyOf(
                 declRefExpr(to(functionDecl().bind("function"))),
                 memberExpr(hasDeclaration(functionDecl().bind("function")))))),
             *body_part, func->getASTContext())) {
      const auto *fn = node.getNodeAs<clang::FunctionDecl>("function");
      callees.insert(fn->getCanonicalDecl());
    }
    for (const auto &node :
         match(findAll(cxxConstructExpr().bind("cxx_construct")), *body_part,
               func->getASTContext())) {
      const auto *ctor_exp =
          node.getNodeAs<clang::CXXConstructExpr>("cxx_construct");
      if (auto ctor = ctor_exp->getConstructor()) {
        callees.insert(ctor);
      }
    }
  }

  return std::move(callees);
}

// Looks for `func` in the `visited_call_stack`. If found it marks `func` and
// each function that came after it as being part of the cycle. This marking is
// stored in the `VisitedCallStackEntry`.
bool FindAndMarkCycleWithFunc(
    llvm::SmallVectorImpl<VisitedCallStackEntry> &visited_call_stack,
    const clang::FunctionDecl *func) {
  // We look for recursive cycles in a simple (but potentially slow for huge
  // call graphs) way. If we reach a function that is already on the call stack
  // (i.e. in `visited`), we declare `func`, and every other function after
  // where `func` was seen in `visited` as being part of a cycle. Then a cycle
  // graph is a contiguous set of functions in the `visited` call stack that are
  // marked as being in a cycle.
  debug("FindAndMarkCycleWithFunc");
  bool found_cycle = false;
  for (size_t i = visited_call_stack.size(); i > 0; --i) {
    const auto &stack_entry = visited_call_stack[i - 1];
    if (stack_entry.func == func) {
      found_cycle = true;
      for (; i <= visited_call_stack.size(); ++i) {
        auto &mut_stack_entry = visited_call_stack[i - 1];
        mut_stack_entry.in_cycle = true;
      }
      break;
    }
  }
  return found_cycle;
}

llvm::SmallVector<const clang::FunctionDecl *>
GetAllFunctionDefinitions(const clang::TranslationUnitDecl *tu) {
  using clang::ast_matchers::findAll;
  using clang::ast_matchers::functionDecl;
  using clang::ast_matchers::hasBody;
  using clang::ast_matchers::isDefinition;
  using clang::ast_matchers::match;
  using clang::ast_matchers::stmt;

  llvm::SmallVector<const clang::FunctionDecl *> functions;

  // For now we specify 'hasBody' to skip functions that don't have a body and
  // are not called. TODO(veluca): a function might be used in other ways.
  for (const auto &node : match(
           findAll(functionDecl(isDefinition(), hasBody(stmt())).bind("func")),
           tu->getASTContext())) {
    const auto *func = node.getNodeAs<clang::FunctionDecl>("func");
    assert(func);
    functions.push_back(func);
  }

  return functions;
}

BaseToOverrides BuildBaseToOverrides(const clang::TranslationUnitDecl *tu) {
  BaseToOverrides base_to_overrides;
  for (const clang::FunctionDecl *f : GetAllFunctionDefinitions(tu)) {
    auto *func = clang::dyn_cast<clang::CXXMethodDecl>(f);
    if (!func)
      continue;
    func = func->getCanonicalDecl();
    if (!func->isVirtual())
      continue;
    for (const auto *base : func->overridden_methods()) {
      base_to_overrides[base->getCanonicalDecl()].insert(func);
    }
  }
  return base_to_overrides;
}

void GetBaseMethods(const clang::CXXMethodDecl *cxxmethod,
                    llvm::DenseSet<const clang::CXXMethodDecl *> &bases) {
  if (cxxmethod->size_overridden_methods() == 0) {
    // TODO(kinuko): It is not fully clear if one method may ever have multiple
    // base methods. If not this can simply return a single CXXMethodDecl rathr
    // than a set.
    bases.insert(cxxmethod);
    return;
  }
  for (const auto *base : cxxmethod->overridden_methods()) {
    // Each method's overridden_methods() only returns an immediate base but not
    // ancestors of further than that, so recursively call it.
    GetBaseMethods(base, bases);
  }
}

std::optional<FunctionLifetimes> GetFunctionLifetimesFromAnalyzed(
    const clang::FunctionDecl *canonical_func,
    const llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
        &analyzed) {
  auto found = analyzed.find(canonical_func);
  if (found == analyzed.end())
    return std::nullopt;
  auto *lifetimes = std::get_if<FunctionLifetimes>(&found->second);
  if (!lifetimes)
    return std::nullopt;
  return *lifetimes;
}

// Update the function lifetimes of `func` with its immediate `overrides` so
// that the lifetimes of the base method will become least permissive. The
// updates will be reflected from the base to its final overrides as this is
// recursively called.
llvm::Error UpdateFunctionLifetimesWithOverrides(
    const clang::FunctionDecl *func,
    llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
        &analyzed,
    const llvm::SmallPtrSet<const clang::CXXMethodDecl *, 2> &overrides) {
  const auto *canonical = func->getCanonicalDecl();
  const auto *method = clang::dyn_cast<clang::CXXMethodDecl>(func);
  assert(method != nullptr);
  assert(method->isVirtual());
  static_cast<void>(method);

  auto opt_lifetimes = GetFunctionLifetimesFromAnalyzed(canonical, analyzed);
  if (!opt_lifetimes)
    return llvm::Error::success();
  FunctionLifetimes base_lifetimes = *opt_lifetimes;

  assert(base_lifetimes.IsValidForDecl(func));

  for (const auto *overriding : overrides) {
    if (overriding->getNumParams() != func->getNumParams()) {
      llvm::errs() << "Param number mismatches between "
                   << method->getParent()->getNameAsString() << " and "
                   << overriding->getParent()->getNameAsString() << "\n";
      func->dump();
      overriding->dump();
      return llvm::createStringError(
          llvm::inconvertibleErrorCode(),
          absl::StrFormat("Param number mismatches between %s and %s\n",
                          method->getParent()->getNameAsString(),
                          overriding->getParent()->getNameAsString()));
    }
    auto opt_override_lifetimes = GetFunctionLifetimesFromAnalyzed(
        overriding->getCanonicalDecl(), analyzed);
    if (!opt_override_lifetimes)
      continue;
    FunctionLifetimes override_lifetimes = *opt_override_lifetimes;

    if (llvm::Error err = ConstrainLifetimes(
            base_lifetimes, override_lifetimes.ForOverriddenMethod(method))) {
      return err;
    }
  }
  analyzed[canonical] = base_lifetimes;
  return llvm::Error::success();
}

llvm::Error AnalyzeRecursiveFunctions(
    llvm::ArrayRef<VisitedCallStackEntry> funcs,
    llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
        &analyzed,
    const DiagnosticReporter &diag_reporter, FunctionDebugInfoMap *debug_info) {
  for (const auto [func, in_cycle, _] : funcs) {
    assert(in_cycle);

    // Grab the initial FunctionLifetimes for each function in the cycle,
    // without doing a dataflow analysis, which would need other functions
    // in the cycle to already be analyzed.
    auto func_lifetimes_result = ConstructFunctionLifetimes(
        func,
        FunctionAnalysis{
            .object_repository = ObjectRepository(func),
        },
        diag_reporter);
    if (!func_lifetimes_result) {
      return func_lifetimes_result.takeError();
    }
    analyzed[func->getCanonicalDecl()] = func_lifetimes_result.get();
  }

  int64_t expected_iterations = 0;
  for (const auto [func, _1, _2] : funcs) {
    expected_iterations =
        std::max(expected_iterations, int64_t{func->getNumParams()});
  }
  // Add 1 for the last iteration that sees nothing changed.
  expected_iterations += 1;

  // Analyze all lifetimes in the cycle repeatedly with dataflow analysis
  // until they stabilize.
  bool func_lifetimes_changed = true;
  for (int64_t count = 0; func_lifetimes_changed; ++count) {
    func_lifetimes_changed = false;

    if (count > expected_iterations) {
      return llvm::createStringError(
          llvm::inconvertibleErrorCode(),
          absl::StrFormat("Recursive cycle requires more than the expected "
                          "%u iterations to resolve!",
                          expected_iterations));
    }

    for (const auto [func, in_cycle, _] : funcs) {
      auto analysis_result =
          AnalyzeSingleFunction(func, analyzed, diag_reporter, debug_info);
      if (!analysis_result) {
        return analysis_result.takeError();
      }
      auto func_lifetimes_result = ConstructFunctionLifetimes(
          func, std::move(analysis_result.get()), diag_reporter);
      if (!func_lifetimes_result) {
        return func_lifetimes_result.takeError();
      }
      // TODO(danakj): We can avoid this structural comparison and just do a
      // check for equality if AnalyzeSingleFunction would reuse Lifetimes
      // from the existing FunctionLifetime for its parameters/return/this.
      // Currently it makes a new set of Lifetimes each time we do the analyze
      // step, but the actual Lifetime ids aren't meaningful, only where and
      // how often a given Lifetime repeats is meaningful.
      FunctionLifetimesOrError &existing_result =
          analyzed[func->getCanonicalDecl()];
      if (std::holds_alternative<FunctionLifetimes>(existing_result) &&
          !IsIsomorphic(std::get<FunctionLifetimes>(existing_result),
                        func_lifetimes_result.get())) {
        existing_result = func_lifetimes_result.get();
        func_lifetimes_changed = true;
      }
    }
  }

  return llvm::Error::success();
}

// * this function analyzes the lifetimes of a given function in the context of
// * its caller functions

// The entry point for analyzing a function named by `func`.
//
// This function is recursive as it searches for and walks through all CallExpr
// instances, calling this function again for each function. This is done to
// analyze the leaves of the call graph first, so that when analyzing a given
// function, all the functions it calls have already been analyzed.
//
// This function also handles walking through recursive cycles of function
// calls. When a cycle is detected, we:
// 1. Do not analyze any of the functions until the cycle is fully explored and
//    we've returned to the entry point to the cycle.
// 2. At that point, we generate a FunctionLifetimes for each function in the
//    cycle, where the lifetimes are all completely disconnected.
// 3. Then we analyze each function in the cycle based on those
//    FunctionLifetimes, connecting lifetimes within the body of each function.
//    This changes a given function's resulting FunctionLifetimes, which can
//    affect the callers to it.
// 4. Thus we repeat step 3 until we see that the FunctionLifetimes have stopped
//    changing when we analyze each function in the cycle.
void AnalyzeFunctionRecursive(
    llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
        &analyzed,
    llvm::SmallVectorImpl<VisitedCallStackEntry> &visited,
    const clang::FunctionDecl *func,
    const LifetimeAnnotationContext &lifetime_context,
    const DiagnosticReporter &diag_reporter, FunctionDebugInfoMap *debug_info,
    const BaseToOverrides &base_to_overrides) {
  // Make sure we're always using the canonical declaration when using the
  // function as a key in maps and sets.
  // ? what is the canonical function declaration
  func = func->getCanonicalDecl();

  // See if we have finished analyzing the function.
  bool is_analyzed = analyzed.count(func) > 0;

  auto *cxxmethod = clang::dyn_cast<clang::CXXMethodDecl>(func);
  bool is_virtual = cxxmethod != nullptr && cxxmethod->isVirtual();
  bool is_pure_virtual = is_virtual && cxxmethod->isPure();

  // + a built in function is a pre-defined function in a programming language
  // + and consequently we do not want to analyze it
  if (func->getBuiltinID() != 0) {
    return;
  }

  if (!func->isDefined() && !is_pure_virtual && !is_analyzed) {
    FunctionLifetimes annotations;
    if (llvm::Error err = GetLifetimeAnnotations(func, lifetime_context)
                              .moveInto(annotations)) {
      analyzed[func] = FunctionAnalysisError(err);
    } else {
      analyzed[func] = annotations;
    }
    return;
  }

  // Check if we're in an overrides traversal for a virtual method.
  bool in_overrides_traversal =
      visited.empty() ? false : visited.back().in_overrides_traversal;

  if (is_analyzed && !in_overrides_traversal) {
    // This function is already analyzed and this analysis is not for an
    // overrides traversal (where repeated update may happen).
    // TODO(kinuko): Avoid repeatedly visit the same virtual methods again and
    // again if all the methods in the same overriding chain are already
    // analyzed.
    return;
  }

  if (!in_overrides_traversal && FindAndMarkCycleWithFunc(visited, func)) {
    // Defer analyzing the cycle until we have fully explored the recursive
    // cycle graph.
    // This cycle check should exclude in_overrides_traversal case, because the
    // traversal can come back to the same function while traversing from its
    // overridden base method, e.g. when we see Child::f() we start the analysis
    // from its overridden implementation Base::f() and then recursively look
    // into its overrides until it reaches its final overrides (and it should
    // see Child::f() on its way.

    // TODO(kinuko): We may return here when Base::f() calls f() even when
    // it has overrides, and if it happens AnalyzeRecursiveFunctions don't
    // look into the overrides so the Base::f() lifetime is not updated.
    // See DISABLED_FunctionVirtualInheritanceWithComplexRecursion tests.
    return;
  }

  // * methods called by the main function
  // + the main function is the one being analyzed

  auto maybe_callees = GetCallees(func);
  if (!maybe_callees) {
    analyzed[func] = FunctionAnalysisError(maybe_callees.takeError());
    return;
  }

  // Keep track of where `func` is found in the call stack. It may not be at the
  // top anymore after we return from calling `AnalyzeFunctionRecursive()` if
  // `func` is part of a recursive cycle, as we keep all members of the
  // recursive cycle in the `visited` stack until we explore the whole graph and
  // then analyze it all.
  size_t func_in_visited = visited.size();
  visited.emplace_back(VisitedCallStackEntry{
      .func = func, .in_cycle = false, .in_overrides_traversal = false});

  for (auto &callee : maybe_callees.get()) {
    // * if the callee function is already analyzed, skip it
    if (analyzed.count(callee)) {
      continue;
    }
    AnalyzeFunctionRecursive(analyzed, visited, callee, lifetime_context,
                             diag_reporter, debug_info, base_to_overrides);
  }

  llvm::DenseSet<const clang::CXXMethodDecl *> bases;
  llvm::SmallPtrSet<const clang::CXXMethodDecl *, 2> overrides;

  // This is a virtual method and we want to recursively analyze the inheritance
  // chain and update the base methods with their overrides. The base methods
  // may be visited and updated repeatedly.
  if (is_virtual) {
    assert(cxxmethod != nullptr);
    visited[func_in_visited].in_overrides_traversal = true;
    if (!in_overrides_traversal) {
      // If it's a virtual method and we are not yet in an overrides traversal,
      // start from the base method.
      GetBaseMethods(cxxmethod, bases);
      for (const auto *base : bases) {
        AnalyzeFunctionRecursive(analyzed, visited, base, lifetime_context,
                                 diag_reporter, debug_info, base_to_overrides);
      }
    } else {
      // We are in an overrides traversal for a virtual method starting from its
      // base method. Recursively look into the overrides that this TU knows
      // about, so that the base method's analysis result can be updated with
      // the overrides (that are discovered in this TU).
      auto iter = base_to_overrides.find(cxxmethod->getCanonicalDecl());
      if (iter != base_to_overrides.end()) {
        overrides = iter->second;
        for (const auto *derived : overrides) {
          AnalyzeFunctionRecursive(analyzed, visited, derived, lifetime_context,
                                   diag_reporter, debug_info,
                                   base_to_overrides);
        }
      }
    }
    visited[func_in_visited].in_overrides_traversal = false;
  }

  // Recursing through CallExprs should not remove `func` from the stack, though
  // there may be more on the stack after `func` if they are all part of a
  // recursive cycle graph.
  assert(visited[func_in_visited].func == func);
  if (func_in_visited < visited.size() - 1) {
    // * this means that func is not at the top of the stack anymore
    // + therefore there is a cycle
    // + find all functions in the stack after func and check that all of them
    // + (including the current function) are in a cycle, otherwise error
    // + the explanation of why this happens is below

    for (size_t i = func_in_visited; i < visited.size(); ++i) {
      assert(visited[i].in_cycle);
    }
  }

  // Once we return back here, there are 3 possibilities for `func`.
  //
  // 1. If `func` is part of a cycle, but was not the first entry point of the
  //    cycle, then we defer analyzing `func` until we get back to the entry
  //    point. We look for this by seeing if there is another function marked as
  //    being in a cycle above `func` in the `visited` call stack. Note that we
  //    will leave `func` in the `visited` call stack when we return so that
  //    once we get back to the recursive cycle's entry point, we can see all
  //    the functions that are part of the cycle graph.
  // ! 2. If `func` was not part of a cycle, we can analyze it and expect it to
  // !   have valid FunctionLifetimes already generated for anything it calls.
  // 3. Otherwise, we collect the whole cycle (which may be just the `func` if
  //    it calls itself directly), and we analyze the cycle as a whole.
  // + this is the case where func is the first entry point of the cycle

  if (func_in_visited > 0 && visited[func_in_visited].in_cycle &&
      visited[func_in_visited - 1].in_cycle) {
    // Case 1. In a recursive cycle, but not the entry point.
    return;
  }
  if (!visited[func_in_visited].in_cycle) {
    // Case 2. Not part of a cycle.
    if (bases.empty()) {
      // This function is not where we initiated an overrides traversal from its
      // base methods.
      auto analysis_result =
          AnalyzeSingleFunction(func, analyzed, diag_reporter, debug_info);
      if (!analysis_result) {
        analyzed[func] = FunctionAnalysisError(analysis_result.takeError());
      } else {
        auto func_lifetimes_result = ConstructFunctionLifetimes(
            func, std::move(analysis_result.get()), diag_reporter);
        if (!func_lifetimes_result) {
          analyzed[func] =
              FunctionAnalysisError(func_lifetimes_result.takeError());
        } else {
          analyzed[func] = func_lifetimes_result.get();
        }
      }
    } else {
      // In this branch we have initiated (and finished) an overrides
      // traversal starting with its base method, and the traversal for this
      // function must be already done as a part of the overrides traversal.
      assert(is_virtual);
      assert(analyzed.count(func) > 0);
    }
  } else {
    // Case 3. The entry point to a recursive cycle.
    auto funcs_in_cycle =
        llvm::ArrayRef<VisitedCallStackEntry>(visited).drop_front(
            func_in_visited);
    if (llvm::Error err = AnalyzeRecursiveFunctions(
            funcs_in_cycle, analyzed, diag_reporter, debug_info)) {
      for (const auto [func_in_cycle, _1, _2] : funcs_in_cycle) {
        analyzed[func_in_cycle] = FunctionAnalysisError(err);
      }
    }
  }

  // If this has overrides and we're in an overrides traversal, the lifetimes
  // need to be (recursively) updated with the results of the overrides.
  if (in_overrides_traversal) {
    if (llvm::Error err =
            UpdateFunctionLifetimesWithOverrides(func, analyzed, overrides)) {
      analyzed[func] = FunctionAnalysisError(err);
    }
  }

  // Once we have finished analyzing `func`, we can remove it from the visited
  // stack, along with anything it called in a recursive cycle (which will be
  // found after `func` in the `visited` call stack.
  visited.resize(func_in_visited);
}

// * Analyze the lifetime of functions in tu
// * Collect the analysis results in a dense map

llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
AnalyzeTranslationUnitAndCollectTemplates(
    const clang::TranslationUnitDecl *tu,
    const LifetimeAnnotationContext &lifetime_context,
    const DiagnosticReporter &diag_reporter, FunctionDebugInfoMap *debug_info,
    llvm::DenseMap<clang::FunctionTemplateDecl *, const clang::FunctionDecl *>
        &uninstantiated_templates,
    const BaseToOverrides &base_to_overrides) {
  llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError> result;
  llvm::SmallVector<VisitedCallStackEntry> visited;

  debug("All function definitions");
  for (const clang::FunctionDecl *func : GetAllFunctionDefinitions(tu)) {
    std::cout << func << std::endl;

    // * uninstantiated_templates -> keep track of templates that have not yet
    // * been instantiated. Below, if one specialization is found, remove it
    // * from the `uninstantiated_templates` map
    // Skip templated functions.
    if (func->isTemplated()) {
      clang::FunctionTemplateDecl *template_decl =
          func->getDescribedFunctionTemplate();
      if (template_decl) {
        uninstantiated_templates.insert({template_decl, func});
      }
      continue;
    }

    if (func->isFunctionTemplateSpecialization()) {
      auto *info = func->getTemplateSpecializationInfo();
      uninstantiated_templates.erase(info->getTemplate());
    }

    // For some reason that's not clear to mboehme@, the AST matcher is
    // returning two matches for every function definition; maybe there are two
    // different paths from a TranslationUnitDecl to a function definition.
    // This doesn't really have any ill effect, however, as
    // AnalyzeFunctionRecursive() bails out anyway if it has analyzed the
    // function before.

    AnalyzeFunctionRecursive(result, visited, func, lifetime_context,
                             diag_reporter, debug_info, base_to_overrides);
  }

  return result;
}

std::string GetFunctionUSRString(const clang::Decl *func) {
  llvm::SmallString</*inline size=*/128> usr;
  if (clang::index::generateUSRForDecl(func, usr)) {
    llvm::errs() << "Could not generate USR for ";
    func->dump();
    assert(false);
    return std::string();
  }
  return std::string(usr.data(), usr.size());
}

// Run AnalyzeFunctionRecursive with `context`. Report results through
// `result_callback` and update `debug_info` using USR strings to map functions
// to the original ASTContext.
void AnalyzeTemplateFunctionsInSeparateASTContext(
    const LifetimeAnnotationContext &lifetime_context,
    const llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
        &initial_result,
    const FunctionAnalysisResultCallback &result_callback,
    const DiagnosticReporter &diag_reporter, FunctionDebugInfoMap *debug_info,
    const std::map<std::string, const clang::FunctionDecl *>
        &template_usr_to_decl,
    const BaseToOverrides &base_to_overrides, clang::ASTContext &context) {
  llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
      inner_result;
  llvm::SmallVector<VisitedCallStackEntry> inner_visited;
  FunctionDebugInfoMap inner_debug_info;

  for (const clang::FunctionDecl *func :
       GetAllFunctionDefinitions(context.getTranslationUnitDecl())) {
    // Skip templated functions.
    if (func->isTemplated())
      continue;

    AnalyzeFunctionRecursive(inner_result, inner_visited, func,
                             lifetime_context, diag_reporter, &inner_debug_info,
                             base_to_overrides);
  }

  // We need to remap the results with FunctionDecl* in the
  // original ASTContext. (Because this context goes away after
  // this)
  llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
      merged_result = initial_result;
  for (const auto &[decl, lifetimes_or_error] : inner_result) {
    if (!decl->isFunctionTemplateSpecialization())
      continue;
    auto *tmpl = decl->getTemplateSpecializationInfo()->getTemplate();
    auto iter = template_usr_to_decl.find(GetFunctionUSRString(tmpl));
    if (iter != template_usr_to_decl.end()) {
      merged_result.insert({iter->second, lifetimes_or_error});
    }
  }
  for (const auto &[decl, lifetimes_or_error] : merged_result) {
    result_callback(decl, lifetimes_or_error);
  }
  for (auto &[decl, info] : inner_debug_info) {
    if (!decl->isFunctionTemplateSpecialization())
      continue;
    auto *tmpl = decl->getTemplateSpecializationInfo()->getTemplate();
    auto iter = template_usr_to_decl.find(GetFunctionUSRString(tmpl));
    if (iter != template_usr_to_decl.end())
      (*debug_info)[iter->second] = info;
  }
}

DiagnosticReporter
DiagReporterForDiagEngine(clang::DiagnosticsEngine &diag_engine) {
  return
      [&diag_engine](clang::SourceLocation location, clang::StringRef message,
                     clang::DiagnosticIDs::Level level) {
        return diag_engine.Report(
            location,
            diag_engine.getDiagnosticIDs()->getCustomDiagID(level, message));
      };
}

} // namespace

bool IsIsomorphic(const FunctionLifetimes &a, const FunctionLifetimes &b) {
  return LifetimeConstraints::ForCallableSubstitution(a, b)
             .AllConstraints()
             .empty() &&
         LifetimeConstraints::ForCallableSubstitution(b, a)
             .AllConstraints()
             .empty();
}

FunctionLifetimesOrError
AnalyzeFunction(const clang::FunctionDecl *func,
                const LifetimeAnnotationContext &lifetime_context,
                FunctionDebugInfo *debug_info) {
  llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
      analyzed;
  llvm::SmallVector<VisitedCallStackEntry> visited;
  std::optional<FunctionDebugInfoMap> debug_info_map;
  if (debug_info) {
    debug_info_map.emplace();
  }
  DiagnosticReporter diag_reporter =
      DiagReporterForDiagEngine(func->getASTContext().getDiagnostics());
  AnalyzeFunctionRecursive(
      analyzed, visited, func, lifetime_context, diag_reporter,
      debug_info_map ? &debug_info_map.value() : nullptr, BaseToOverrides());
  if (debug_info) {
    *debug_info = debug_info_map->lookup(func);
  }
  return analyzed.lookup(func);
}

// ! The function below is called in the file
// ! lifetime_analysis_test.cc

llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
AnalyzeTranslationUnit(const clang::TranslationUnitDecl *tu,
                       const LifetimeAnnotationContext &lifetime_context,
                       DiagnosticReporter diag_reporter,
                       FunctionDebugInfoMap *debug_info) {
  // * tu -> represents the code to be analyzed
  // * lifetime_context -> contains information about the lifetime annotations

  if (!diag_reporter) {
    diag_reporter =
        DiagReporterForDiagEngine(tu->getASTContext().getDiagnostics());
  }

  // * this is used for function templates
  llvm::DenseMap<clang::FunctionTemplateDecl *, const clang::FunctionDecl *>
      uninstantiated_templates;

  // * Builds a map from a base method to its overrides within this TU.
  // ! It will not find out all the overrides,
  // but still cover (and can partially update) all the base methods that this
  // TU implements.
  auto base_to_overrides = BuildBaseToOverrides(tu);

  // * Map that associates each function declaration with the respective
  // * lifetimes
  llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError> result =
      AnalyzeTranslationUnitAndCollectTemplates(
          tu, lifetime_context, diag_reporter, debug_info,
          uninstantiated_templates, base_to_overrides);

  return result;
}

// ! The function below is called in the file
// ! lifetime_analysis_test.cc

void AnalyzeTranslationUnitWithTemplatePlaceholder(
    const clang::TranslationUnitDecl *tu,
    const LifetimeAnnotationContext &lifetime_context,
    const FunctionAnalysisResultCallback &result_callback,
    DiagnosticReporter diag_reporter, FunctionDebugInfoMap *debug_info) {
  if (!diag_reporter) {
    diag_reporter =
        DiagReporterForDiagEngine(tu->getASTContext().getDiagnostics());
  }

  llvm::DenseMap<clang::FunctionTemplateDecl *, const clang::FunctionDecl *>
      uninstantiated_templates;

  // Builds a map from a base method to its overrides within this TU. It will
  // not find out all the overrides, but still cover (and can partially update)
  // all the base methods that this TU implements.
  auto base_to_overrides = BuildBaseToOverrides(tu);

  // * map that associates each function declaration with the respective
  // * lifetimes
  llvm::DenseMap<const clang::FunctionDecl *, FunctionLifetimesOrError>
      initial_result = AnalyzeTranslationUnitAndCollectTemplates(
          tu, lifetime_context, diag_reporter, debug_info,
          uninstantiated_templates, base_to_overrides);

  // Make a map from USRString to funcDecls in the original ASTContext.
  std::map<std::string, const clang::FunctionDecl *> template_usr_to_decl;
  for (const auto &[tmpl, func] : uninstantiated_templates) {
    template_usr_to_decl[GetFunctionUSRString(tmpl)] = func;
  }

  GeneratedCode code_with_placeholder;
  if (llvm::Error err =
          GenerateTemplateInstantiationCode(tu, uninstantiated_templates)
              .moveInto(code_with_placeholder)) {
    FunctionAnalysisError analysis_error(err);
    for (const auto &[tmpl, func] : uninstantiated_templates) {
      result_callback(func, analysis_error);
    }
    return;
  }

  // A callback to call AnalyzeFunctionRecursive again with template
  // placeholders. This is passed to RunToolOnCodeWithOverlay below.
  auto analyze_with_placeholder =
      [&lifetime_context, &initial_result, &result_callback, &diag_reporter,
       &debug_info, &template_usr_to_decl,
       &base_to_overrides](clang::ASTContext &context) {
        AnalyzeTemplateFunctionsInSeparateASTContext(
            lifetime_context, initial_result, result_callback, diag_reporter,
            debug_info, template_usr_to_decl, base_to_overrides, context);
      };

  // Run `analyze_with_placeholder` in a separate ASTContext on top of an
  // overlaid filesystem with the `code_with_placeholder` file.
  RunToolOnCodeWithOverlay(tu->getASTContext(), code_with_placeholder.filename,
                           code_with_placeholder.code,
                           analyze_with_placeholder);
}

} // namespace lifetimes
} // namespace tidy
} // namespace clang
