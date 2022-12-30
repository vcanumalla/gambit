## Gambit: Certora's Mutation Generator for Solidity

This is a mutation generator for Solidity.
Mutation Testing is a technique for
  evaluating the quality of and improving test suites or specifications used
  for testing or verifiying Solidity smart contracts.

Gambit traverses the Solidity AST generated by the Solidity compiler
  to detect valid "mutation points"
  and uses the `src` field in the AST to directly mutate the source.

*NOTE*: At the moment, we support simple ASTs that do not require complex build systems.
We are actively working on supporting more complex Solidity projects!

Gambit is implemented in Rust which
you can download from [here](https://www.rust-lang.org/tools/install).

### Users
You can learn how to use Gambit by running
`cargo run --release -- mutate --help`.
It will show you all the command line arguments that Gambit accepts.

As you can see, Gambit accepts a configuration file as input where you can
  specify which files you want to mutate and using which mutations.
You can control which functions and contracts you want to mutate.
Examples of some configuration files can be found under `benchmarks/config-jsons`.

#### Examples of how to run Gambit:
- cargo run --release -- mutate --json benchmarks/config-jsons/test1.json
- cargo run --release -- mutate -f benchmarks/RequireMutation/RequireExample.sol

### Developers
- `cargo build`, `cargo fmt`, `cargo clippy` before pushing.

### Credits
We thank
[Oliver Flatt](https://www.oflatt.com/) and
[Vishal Canumalla](https://homes.cs.washington.edu/~vishalc/)
for their excellent contributions to an earlier prototype of Gambit.