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

## Step x: construct the function lifetimes

1. Set `object repository`, `points-to-map`, `constraints` and `lifetime substituions` to the output of the previously made _analysis_
2. Get the _original_ function lifetimes from the `object repository`
3. apply the lifetime `constraints` to the original function lifetimes
4. Diagnose the result (probably not that important for the pseudocode)
