# Dataflow Analysis

The actual analysis is performed in the function `runTypeErasedDataflowAnalysis`

```
RunDataflowAnalysis(CFG, Analysis, InitEnv) -> BlockStates {
    POV = PostOrderCFGView(CFG)
    Worklist = ForwardWorklist(CFG, POV)    // for forward dataflow analysis
    BlockStates = {} // size = CFG.size


}
```

---

`Clang::Tooling` applies a "frontend action", in this case the _lifetime analysis_ to the code's AST

```
Clang::Tooling::RunAnalysisOnCode(source_code, LifetimeAnalysis, ...)
```

This is a lambda function, applied by `Clang::Tooling` on the code's AST
(corresponds to the lambda function `test`)

```
LifetimeAnalysis(ast_context, lifetime_context) -> tu_lifetimes {
    analysis_result = new Map<func_decl, lifetimes>()
    funcs_visited = new SmallVector()
    all_functions = GetAllFunctionDefinitions(ast_context)

        // perform recursive analysis for each function
        for func in all_functions {
            AnalyzeFunctionRecursive(analysis_result, visited, func)
        }

        for func, lifetimes in analysis_result {
            lifetimes = buildTestOutput(func, lifetimes) // lambda function
        }
    return tu_lifetimes
}

```

- Searches for and walks through all `CallExpr` instances, calling this function again for each function call -> recursive
- This function analyzes the lifetimes of a given function in the context of its caller functions
- Analyzes the _leaves_ of the call graph first, thus, when analyzing a given function, all the functions it calls have already been analyzed
- This also handles walking through recursive cycles of function calls -> skipped in the pseudocode
- The explanation of how this is handled is given below

```
AnalyzeFunctionRecursive(analysis_result, visited, func) {
    // func called by the current one
    callees = GetCallees(func)
    visited.insertBack(func)

    for callee in callees {
        if (callee ∈ analysis_results) return
        AnalyzeFunctionRecursive(analysis_result, visited, callee)
    }

    // check if not in cycle then:
    analysis_result = AnalyzeSingleFunction(func, result, ...)

    analysis_result[func] = ConstructFunctionLifetimes(func, analysis_result)
}

```

Analysis of a single function (not a constructor, not a virtual method) -> function with body

```
struct FunctionAnalysis {
    object_repository
    points_to_map
    lifetime_constraints
    lifetime_substitutions
}

AnalyzeSingleFunction(func, analyzed, ...) -> func_analysis {
    func_analysis = new FunctionAnalysis()
    body = GetBody(func)
    ast_context = GetASTContext(func)

    // only one case covered -> functions with body
    cfg_context = BuildCfgContext(func, body, ast_context)
    analysis_context = new DataflowAnalysisContext()
    environment = new Environment(analysis_context)

    lifetime_lattice_vector = RunDataflowAnalysis(cfg_context, analysis_context, environment)

    // represents the state of the program after the dataflow analysis is finished
    exit_block = GetExitBlock(lifetime_lattice_vector)

    exit_lattice = GetLattice(exit_block)
    points_to_map = GetPointsTo(exit_lattice)
    constraints = GetConstraints(exit_lattice)

    ExtendStaticConstraint(points_to_map, constraints)

    PropagateStaticToPointees(func_analysis)
    return func_analysis
}
```

"Extra": extend the constraint set with constraints of the form `a >= static`

```
// extend the constraints set with additional constraints of the form "a >= static"
// these constraints means that the lifetime `a` must be at least as long as the `static` lifetime
// the `static` lifetime is the longest, can be considered the root of the lifetime hierarchy
// to do so we need to find all objects that are reachable from a `static` object
// ? what is the meaning of "transitively"
// ? also, there are "outlive" lifetimes afterall?

ExtendStaticConstraint(points_to_map, constraints) {
    // collect all points that have lifetime `static`
    stack = GetAllPointersWithLifetime(points_to_map, Lifetime::Static())
    visited = new Set()
    while (stack.size > 0) {
        object = removeBack(stack)
        if (containsObject(visited, object)) continue
        insert(visited, object)
        // insert {shorter, longer} in the constraints set
        AddOutlivesConstraint(constraints, Lifetime::Static(), GetLifetime(object))

        for pointee in GetPointersPointsToSet(object) {
            insertBack(pointee, stack)
        }
    }
}
```

Construct the lifetime annotations

```
ConstructFunctionLifetimes(func, analysis_result) -> lifetime_result {
    result = GetOriginalFunctionLifetimes(object_repository)
    result = ApplyToFunctionLifetimes(constraints)
    return result
}

ApplyToFunctionLifetimes(constraints) {
    output_lifetimes = GetOutputLifetimes()
    function_call_lifetimes = GetFunctionCallLifetimes()

    substitutions = LifetimeSubstitutions()
    already_have_substitutions = DenseSet<>()

    // 1. substitute everything that outlives "static" with `static`
    for outlives_static in GetOutlivingLifetimes(Lifetime::Static()) {
        already_have_substitutions.insert(outlives_static)
        substitutions.add(outlives_static, Lifetime::Static())
    }

    

}
```

### Notes

- skipped templates
- skipped virtual methods
- skipped constructors
- skipped cycles

It uses dataflow analysis to find out the `points_to_map` and the `lifetime_constraints`

Then uses the map and the constraints to create the _lifetime annotations_
