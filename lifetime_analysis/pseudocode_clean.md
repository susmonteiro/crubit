# Dataflow Analysis

The actual analysis is performed in the function `runTypeErasedDataflowAnalysis`

```
RunDataflowAnalysis(CFG, Analysis, InitEnv) -> BlockStates {
    POV = PostOrderCFGView(CFG)

    // the enqueued blocks will be dequeued in reverse post order
    // the worklist never contains the same block multiple times at once
    Worklist = ForwardWorklist(CFG, POV)    // for forward dataflow analysis
    BlockStates = {} // size = CFG.size

    while not_empty(worklist) {
        block = Worklist.popFromTop()
        old_block_state = BlockStates[block]
        new_block_state = transferCFGBlock(block, Analysis)

        if (old_block_state == new_block_state) continue    // no need to visit sucessors

        BlockStates[block] = new_block_state

        Worklist.addSuccessors(block)
    }

    return BlockStates
}

TransferCFGBlock(block, Analysis) {
    State = ComputeBlockInputState(Block, Analysis)

    for Element in Block {
        // skips BuiltinTransfer --> `lifetime_analysis.h`
        State = SpecificTransfer(Element, Analysis, State)
        // skips PostVisitCFG --> `analyze.cc` (the function is not passed as arguments, so it is null by default)
    }
    return State
}

```

Previous two function together:

```
RunDataflowAnalysis(CFG, Analysis, InitEnv) -> BlockStates {
    Worklist = ForwardWorklist(CFG)    // for forward dataflow analysis
    BlockStates = {} // size = CFG.size

    while not_empty(worklist) {
        block = Worklist.popFromTop()

        State = ComputeBlockInputState(Block, Analysis)
        for Element in Block {
            State = LifetimeAnalysisTransfer(Element, Analysis, State)
        }

        if (BlockStates[block] == State) continue    // no need to visit sucessors
        BlockStates[block] = State
        Worklist.addSuccessors(block)
    }

    return BlockStates
}
```

The function `transferCFGBlock` (in the function `runTypeErasedDataFlowAnalysis`) calls a DataflowAnalysis function (`TransferTypeErased`) which then calls the specific analysis `transfer` function.
This is not shown in the pseudo code.

State holds two things: **environment** and **lattice**

- **Environment**: holds the state of the program (store and heap) at a given program point
- **Lattice**: holds the points_to_map and the constraints

Lifetime Analysis class. Files: `lifetime_analysis.h` and `lifetime_analysis.cc`

```

// new pointees must always outlive the old pointees
GenerateConstraintsForAssignmentNonRecursive(old_pointees, new_pointees, constraints) {
    for old in old_pointees {
        for new in new_pointees {
            constraints.addOutlivesConstraint(GetLifetime(old), GetLifetime(new))
        }
    }
}

GenerateConstraintsForAssignmentRecursive(pointers, new_pointees, pointer_type, object_repository, points_to_map, constraints, seen_pairs) {

    // all pairs were already seen -> check for cycles
    if (size(seen_pairs) == combinations(pointers, new_pointees)) return

    old_pointees = points_to_map.GetPointerPointsToSet(pointers)
    GenerateConstraintsForAssignmentNonRecursive(old_pointees, new_pointees, constraints)

    // skip recursive part -> needed for structs, whose fields may be pointees
}

GenerateConstraintsForAssignment(pointers, new_pointees, pointer_type, object_repository, points_to_map, constraints) {
    seen_pairs = DenseSet<Pair>

    GenerateConstraintsForAssignmentRecursive(pointers, new_pointees, pointer_type, object_repository, points_to_map, constraints, seen_pairs)
}



```

This is the transfer function for each kind of statement:

- `VisitExpr`: do nothing
- `VisitReturnStmt`: only need to handle pointers and references
  ```
  if (isPointerType(return) or (isReferenceType(return))) {
      expr_points_to = points_to_map.GetExprObjectSet(return)
      GenerateConstraintsForAssignment()
  }
  ```
- `VisitUnaryOperator`:

  - AddrOf (`&`): add the corresponding object to the `expr_objects_[expr]` of the `points_to_map`
  - Deref (`*`): same as AddrOf, but assert the opposite
  - PostInc/PostDec
  - PreInc/PostInc
  - The rest: do nothing

- `VisitBinaryOperator`:

  - Assign (`=`):

  ```
  input:
    op -> assign operator
    lhs, rhs
  begin:
    lhs_points_to = points_to_map.GetExprObjectSet(lhs)
    points_to_map.SetExprObjectSet(op, lhs_points_to)

    if (PointerType(lhs)) {
        rhs_points_to = points_to_map.GetExprObjectSet(rhs)
        // lhs points to all rhs pointers
        points_to_map.SetPointerPointsToSet(lhs_points_to, rhs_points_to)
    }
  ```

  - Add/Sub (`+`/`-`): consider only pointer arithmetic (only one of the operands is a pointer). If this is the case, add that pointer to the points_to_map. Otherwise, do nothing

- `VisitCallExpr`:

  - finds all possible callees
  - for each callee:
    - for each param in the declaration, find the points-to-set of that param and assigns it to the arg object in the "parent" function
    - find the points-to-set of the return value and assigns it to the call object

  ```
  callees = EmptyVector()
  direct_callee = getDirectCallee(call)

  if (direct_callee) {
      // skip builtin
      callee_lifetimes = GetLifetimes(direct_callee)

      // skip member functions
      callees.push_back(callee_lifetimes)
  } else {
    // skip function pointer calls and virtual calls
    // in this case, determine the callees by analyzing the possible objects that the callee could point to
  }

  for calle in callees {
    for arg in arguments {
        TransferInitializer(
            object_repository_.GetCallExprArgumentObject(call, i),
            callee.type,
            object_repository,
            arg,
            points_to_map,
            constraints
        )
    }
    // skip member functions
  }

  if (!RecordObject(call)) {
    return_points = points_to_map.GetPointerPointsToSet(call)
    if (NotEmpty(return_points)) {
        points_to_map.SetExprObjectSet(call, return_points)
    }
  }

  ```

  `TransferInitializer` gets the points-to of the ith target function's argument and gives it to the ith call arg object

  ```
  TransferInitializer(destination, type, object_repository, init_expr, points_to_map, constraints) {
    if (isArrayType(type)) {
        // do something
    }
    if (isRecordType(type)) {
        // do something
    }
    if (isPointerType(type) or isReferenceType(type) or isStructureOrClassType(type)) {
        init_points_to = points_to_map.GetExprObjectSet(init_expr) // get points-to of the arg
        points_to_map.SetPointerPointsToSet(destination, init_points_to)
    }
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
