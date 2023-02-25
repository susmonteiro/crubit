### ValueLifetimes:

Class that represents the lifetimes of a value

- specification:
  - non-reference-like types -> 0
  - pointers/references -> 1
  - structs with template arguments/
    lifetime parameters -> arbitrary
- lifetimes are created in **post-order** in the tree of lifetimes

### Object repository:

A repository for objects used in the lifetime analysis of a single function

- relationship between AST nodes (e.g. variable declarations) and the objects that represent them
- stores additional information about obkects that does not change during the analysis
- not part of the **lattice** (stores only state that does not change during the analysis)

---

# Steps in the Analysis

## Step x-2:

Function: `AnalyzeFunctionRecursive()`

## Step x-1: analyze recursive functions

Function: `AnalyzeRecursiveFunctions()`

Arguments (important):

- array of functions
- map of analyzed functions (functiondecl -> lifetimes/error)

## Step x: construct the function lifetimes

Function: `ConstructFunctionLifetimes()`

1. Set `object repository`, `points-to-map`, `constraints` and `lifetime substituions` to the output of the previously made _analysis_
2. Get the _original_ function lifetimes from the `object repository`
3. apply the lifetime `constraints` to the original function lifetimes
4. Diagnose the result (probably not that important for the pseudocode)

---

# Pseudocode

## Important files:

- `lifetime_analysis/analyze.cc`
- `lifetime_analysis/lifetime_analysis.cc`
- `lifetime_analysis/lifetime_analysis_test.cc`

## How the tests work

1. Call function `GetLifetimes()` which receives a piece of code and returns the lifetime information related to the name functions in that code.

## Actual Pseudocode

Functions -> PascalCase
Lambda functions -> camelCase
Variables -> snake_case

```
>> Let's take the case where there are no placeholders (most cases)
>> The cases with placeholders are only used in the function_templates.cc tests

GetLifetimes(source_code, options) -> lifetimes {
  // verifies if the analysis was successful and if so returns the correct lifetimes
  lifetimes = RunAnalysisOnCode(source_code, runAnalysis, ...)
  return lifetimes
}

>> This is the lambda function `test`
runAnalysis() {
  analysis_result = new Map<func_decls, lifetimes>()

  AnalyzeTranslationUnit(ast_context, lifetime_context, ...)

  for [func, lifetimes] in analysis_result {
    resultCallback(func, lifetimes) // build the output that is going to be compared in the tests
  }
}

>> This is the lambda function `result_callback`
resultCallback(func, lifetimes) {

}

```

```

>> Not important
AnalyzeTranslationUnitWithTemplatePlaceholder() {
  AnalyzeTemplateFunctionsInSeparateASTContext()

}

<!-- ! Called in the test file lifetime_analysis_test.cc -->
<!-- TODO -->
AnalyzeTranslationUnit() {

}

<!-- * STILL NEED TO DO THIS ONE -->
<!-- TODO should probably start here -->
AnalyzeFunction() -> Lifetimes {

}

<!-- TODO -->
AnalyzeTemplateFunctionsInSeparateASTContext() {

}

<!-- TODO -->
AnalyzeTranslationUnitAndCollectTemplates() {

}

<!-- TODO -->
AnalyzeFunctionRecursive(analyzed_map, visited, func_decl, lifetime_context, ...) {
  (...)
  for (...) {
    AnalyzeFunctionRecursive(...) // recursive call
  }
}

<!-- TODO -->
AnalyzeRecursiveFunctions(functions, analyzed_map, ...) {

}

ConstructFunctionLifetimes(func_decl, function_analysis, ...) -> function_lifetimes {
  result = getOriginalFunctionLifetimes(function_analysis.object_repository)
  applyToFunctionLifetimes(result, function_analysis.constraints) -> error
  diagnoseReturnLocal(func_decl, result, ...) -> error
  return result
}


```
