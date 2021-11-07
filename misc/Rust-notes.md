# Project structure

- Optional dependencies define new implicit features with the same name
- The work for separating this into 2 namespaces is underway
- Cargo will not compile the same dependency twice with different sets of features. Instead, it will compile only once with the union of the features.
- It may be useful to check all possible feature combinations in CI (e.g. using something like cargo-hack) to make sure the features are additive and are not mutually exclusive

- Workspaces are good but there are caveats about interdependencies - need to re-read if I need it

# Project configuration

- Cargo can override dependencies with [patch] temporarily, which may be useful for testing
- There are various options related to publishing
- We can configure how the program behaves on panic (e.g. unwinds or aborts)
- When a thread panics and unwinds, it tries to clean up while being in a half-working state. This may cause problems when doing work in critical sections, and this is why some sync primitives remember if a panic has occurred.
- It's possible to enable full LTO

# Testing

- Mocking in Rust is usually done via generics
- It may be useful to do fuzz-testing of the program. cargo-fuzz is a tool to look at
- Tools like Miri and Loom can help debug issues with UB and data races
- For performance testing it's important not to measure stuff you don't want to measure, e.g. I/O overhead
- If necessary, you can disable some optimisations using the black_box function
- Given that performance very often varies for reasons not related to the code itself, it's important to run benchmarks several times and maybe print a distribution histogram
