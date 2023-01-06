## <img src="doc/gambit.png" alt="gambit logo" height="40" align="left"> Gambit: Certora's Mutation Generator for Solidity

This is a mutation generator for Solidity.
Mutation Testing is a technique for
  evaluating and improving test suites or specifications used
  for testing or verifiying Solidity smart contracts.

Gambit traverses the Solidity AST generated by the Solidity compiler
  to detect valid "mutation points"
  and uses the `src` field in the AST to directly mutate the source.

Gambit takes as input a solidity source file (or a configuration file as you can see below)
  and produces a set of uniquely mutated solidity source files which are, by default, dumped in
  the `out/` directory.


### Requirements

- Gambit is implemented in Rust which you can download from [here](https://www.rust-lang.org/tools/install).
- To run Gambit, do the following:
   - `git clone git@github.com:Certora/gambit.git`
   - Follow the directions [below](#users).
   - As an alternative, you can also _install_ Gambit by running `cargo install --path .` from the `gambit/` directory after you clone the repo.
- You will need OS specific binaries for various versions of solidity. You can download them [here](https://github.com/ethereum/solc-bin).

### Users
You can learn how to use Gambit by running
`cargo gambit-help`.
It will show you all the command line arguments that Gambit accepts.

As you can see, Gambit accepts a configuration file as input where you can
  specify which files you want to mutate and using which mutations.
You can control which functions and contracts you want to mutate.
Examples of some configuration files can be found under `benchmarks/config-jsons`.

#### Examples of how to run Gambit
- `cargo gambit-cfg benchmarks/config-jsons/test1.json`
- `cargo gambit benchmarks/RequireMutation/RequireExample.sol`
- For projects that have complex dependencies and imports, you will likely need to
  pass the `--base-path` argument for `solc` like so:
`cargo gambit path/to/file.sol --solc-basepath base/path/dir/.`.
If you are using a config file, you can also pass this argument there as a field, e.g.,
```
{
  "filename": "path/to/file.sol",
  "solc-basepath": "base/path/dir/."
}
```
For using the other command line arguments, run `cargo gambit-help`.
You can print log messages by setting the environment variable `RUST_LOG` (e.g., `RUST_LOG=info cargo gambit ...`).

#### Output of Gambit
Gambit produces a set of uniquely mutated solidity source files which are, by default, dumped in
  the `out/` directory.
Each mutant file has a comment that describes the exact mutation that was done.
For example, one of the mutant files for `benchmarks/10Power/TenPower.sol` that Gambit generated contains:
```
/// SwapArgumentsOperatorMutation of: uint256 res = a ** decimals;
uint256 res = decimals ** a;
```


### Demo
Here is a demo of Gambit generating mutants for [AaveTokenV3.sol](https://github.com/Certora/aave-token-v3/blob/main/src/AaveTokenV3.sol).
You can clone the Aave repo and then run Gambit with a config file like:

```
{
    "filename": "PATH/TO/aave-token-v3/src/AaveTokenV3.sol",
    "solc-basepath": "PATH/TO/aave-token-v3/.",
    "contract": "AaveTokenV3",
}
```

<img src="doc/gambit-animation.jif" height="450">

### Developers
We are happy to accept contributions to Gambit! A few tips:
- [VSCode](https://code.visualstudio.com/) is a good IDE for Rust development.
- Run `make` before you push --- it will build Gambit and run all the tests.

### Mutation Types
At the moment, Gambit implements the following mutations:
- Binary Operator Mutation: change a binary operator `bop` to `bop'`,
- Unary Operator Mutation: change a unary operator, `uop` to `uop'`,
- Require Condition Mutation: negate the condition,
- Assignment Mutation: change the RHS,
- Delete Expression Mutation: comment out some expression,
- Function Call Mutation: randomly replace a function call with one of its operands,
- If Statement Mutation:  negate the condition,
- Swap Function Arguments Mutation: swap the arguments to a function,
- Swap Operator Arguments Mutation: swap the operands of a binary operator,
- Swap Lines Mutation: swap two lines
- Eliminate Delegate Mutation: replace a delgate call by `call`.

As you can imagine, many of these mutations may lead to invalid mutants
  that do not compile.
At the moment, Gambit simply compiles the mutants and only keeps valid ones --
  we are working on using additional type information to reduce the generation of
  invalid mutants by constructions. 
You can see the implementation details in `mutation.rs`.


### Contact
If you have ideas for intersting mutations or other features,
  we encourage you to make a PR or [email](mailto:chandra@certora.com) us.

### Credits
We thank
[Oliver Flatt](https://www.oflatt.com/) and
[Vishal Canumalla](https://homes.cs.washington.edu/~vishalc/)
for their excellent contributions to an earlier prototype of Gambit.
