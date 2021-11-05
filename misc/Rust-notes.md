# Project structure

- Optional dependencies define new implicit features with the same name
- The work for separating this into 2 namespaces is underway
- Cargo will not compile the same dependency twice with different sets of features. Instead, it will compile only once with the union of the features.
- It may be useful to check all possible feature combinations in CI (e.g. using something like cargo-hack) to make sure the features are additive and are not mutually exclusive

- Workspaces are good but there are caveats about interdependencies - need to re-read if I need it
