/*!
* Workflow:
* Let's mainly focus on the mutation
* generation part for now.
* This tool should take as input, a solidity file,
* then compile it to generate it's AST and do various mutations of it.
* All the mutated files should be in some directory that the user will
* pass from the commandline.
!*/

use clap::{Parser, ValueEnum};
use rand::SeedableRng;
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::BufReader;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

mod ast;
pub use ast::*;
mod mutation;
pub use mutation::*;
mod run;
pub use run::*;
mod util;
pub use util::*;

#[derive(Debug, Clone)]
pub struct MutantGenerator {
    /// Params for controlling the mutants.
    pub params: MutationParams,
    /// will need this for randomization
    pub rng: Pcg64,
}

impl MutantGenerator {
    /// Initialize the MutantGenerator
    pub fn new(params: MutationParams) -> Self {
        MutantGenerator {
            rng: rand_pcg::Pcg64::seed_from_u64(params.seed),
            params,
        }
    }

    /// Compile the input solc files and get json ASTs.
    pub fn compile_solc(&self, sol: &String, out: PathBuf) -> SolAST {
        let norms_to_path = get_path_normals(sol);
        let norm_sol = norms_to_path.to_str().unwrap_or_else(|| {
            panic!("Could not convert the path to the sol file to a normalized version.")
        });
        let sol_path = out.join("input_json/".to_owned() + norm_sol);
        std::fs::create_dir_all(sol_path.parent().unwrap())
            .expect("Unable to create directory for storing input jsons.");
        log::info!(
            "made parent directories for writing the json file at {}.",
            sol_path.to_str().unwrap()
        );
        if invoke_command(
            &self.params.solc,
            vec![
                "--ast-compact-json",
                sol,
                "-o",
                sol_path.to_str().unwrap(),
                "--overwrite",
            ],
        )
        .0
        .unwrap_or_else(|| panic!("solc terminated with a signal."))
            != 0
        {
            panic!("Failed to compile source. Try with a different version of solc.")
        }
        let ast_fnm = Path::new(sol)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
            + "_json.ast";
        let ast_path = sol_path.join(&ast_fnm);
        let json_fnm = sol_path.join(ast_fnm + ".json");
        std::fs::copy(ast_path, &json_fnm).expect("Could not copy .ast content to .json");
        let json_f = File::open(&json_fnm).unwrap_or_else(|_| {
            panic!("Cannot open the json file {}", &json_fnm.to_str().unwrap())
        });
        let ast_json: Value =
            serde_json::from_reader(json_f).expect("AST json is not well-formed.");
        SolAST {
            element: Some(ast_json),
        }
    }

    /// Generate mutations for a single file.
    fn run_one(
        &self,
        file_to_mutate: &String,
        muts: Option<Vec<String>>,
        funcs: Option<FunctionMutationMapping>,
        contract: Option<String>,
    ) {
        let rand = self.rng.clone();
        let outdir = Path::new(&self.params.outdir);
        let ast = self.compile_solc(file_to_mutate, outdir.to_path_buf());
        let mut_types = muts.map_or(MutationType::value_variants().to_vec(), |ms| {
            ms.iter()
                .map(|m| MutationType::from_str(m, true).unwrap())
                .collect()
        });

        let run_mutation = RunMutations {
            fnm: file_to_mutate.into(),
            node: ast,
            num_mutants: self.params.num_mutants,
            rand,
            out: outdir.to_path_buf(),
            mutation_types: mut_types,
            funcs_to_mutate: funcs,
            contract,
        };
        log::info!("running mutations on file: {}", file_to_mutate);

        let is_valid = |mutant: &str| -> bool {
            let tmp_file = "tmp.sol";
            std::fs::write(tmp_file, mutant)
                .expect("Cannot write mutant to temp file for compiling.");
            let (valid, _, _) = invoke_command(&self.params.solc, vec![tmp_file]);
            std::fs::remove_file(tmp_file)
                .expect("Cannot remove temp file made for checking mutant validity.");
            match valid {
                Some(n) => n == 0,
                None => false,
            }
        };

        run_mutation.get_mutations(is_valid);
    }

    fn run_from_config(&mut self, cfg: &String) {
        let f = File::open(cfg).expect("Cannot open json config file.");
        let config: Value = serde_json::from_reader(BufReader::new(f)).expect("Illformed json.");
        let mut process_single_file = |v: &Value| {
            if let Some(filename) = &v.get("filename") {
                let mut func_mut_map = HashMap::new();
                let fnm = filename.as_str().unwrap();
                if let Some(num) = &v.get("num-mutants") {
                    self.params.num_mutants = num.as_i64().unwrap();
                }
                if let Some(solc) = &v.get("solc") {
                    self.params.solc = solc.as_str().unwrap().to_string();
                }
                if let Some(seed) = &v.get("seed") {
                    self.params.seed = seed.as_u64().unwrap();
                }
                let contract: Option<String> =
                    v.get("contract").map(|v| v.as_str().unwrap().to_string());

                if let Some(muts) = &v.get("mutations") {
                    let muts: Vec<String> = muts
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap().to_string())
                        .collect();
                    self.run_one(&fnm.to_string(), Some(muts), None, contract.to_owned());
                }
                if let Some(funcs) = &v.get("functions") {
                    for func in funcs.as_array().unwrap().iter() {
                        func_mut_map.insert(
                            func.get("name")
                                .unwrap()
                                .as_str()
                                .unwrap_or_else(|| panic!("`functions` field must have `name`s"))
                                .to_string(),
                            func.get("mutations")
                                .unwrap_or_else(|| {
                                    panic!(
                                        "`functions` field must have an array `mutations` field."
                                    )
                                })
                                .as_array()
                                .unwrap_or_else(|| panic!("`mutations` must be an array field."))
                                .iter()
                                .map(|n| n.as_str().unwrap().to_string())
                                .collect(),
                        );
                    }
                    self.run_one(&fnm.to_string(), None, func_mut_map.into(), contract);
                } else {
                    self.run_one(&fnm.to_string(), None, None, contract);
                }
            }
        };
        match config {
            Value::Array(elems) => {
                for elem in elems {
                    process_single_file(&elem);
                }
            }
            Value::Object(_) => process_single_file(&config),
            _ => panic!("Ill-formed json probably."),
        }
    }

    /// Main runner
    pub fn run(&mut self) {
        log::info!("starting run()");
        let files = &self.params.filename;
        let json = &self.params.json.clone();
        if files.is_some() {
            for f in files.as_ref().unwrap() {
                self.run_one(f, None, None, None);
            }
        } else if json.is_some() {
            self.run_from_config(json.as_ref().unwrap())
        } else {
            panic!("Must provide either --filename file.sol or --json config.json.")
        }
    }
}

///
/// Command line arguments for running Gambit.
/// Following are the main ways to run it.
///
///    1. cargo run --release -- mutate --filename path/to/file.sol: this will apply all mutations to file.sol.
///
///    2. cargo run --release -- mutate -f path/to/file1.sol -f path/to/file2.sol: this will apply all mutations to file1.sol and file2.sol.
///
///    3. cargo run --release -- mutate --json path/to/config.json: this gives the user finer control on what functions in
///       which files, contracts to mutate using which types of mutations.
#[derive(Debug, Clone, Parser, Deserialize, Serialize)]
#[command(rename_all = "kebab-case")]
pub struct MutationParams {
    /// Json file with config
    #[arg(long, short, conflicts_with = "filenames")]
    pub json: Option<String>,
    /// Files to mutate
    #[arg(long, short, conflicts_with = "json")]
    pub filename: Option<Vec<String>>,
    /// Num mutants
    #[arg(long, short, default_value = "5")]
    pub num_mutants: i64,
    /// Directory to store all mutants
    #[arg(long, short, default_value = "out")]
    pub outdir: String,
    /// Seed for random number generator
    #[arg(long, short, default_value = "0")]
    pub seed: u64,
    /// Solidity compiler version
    #[arg(long, default_value = "solc")]
    pub solc: String,
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Command {
    Mutate(MutationParams), // Maybe we want to do other things in the future like support checking mutants?
}

/// Entry point
fn main() {
    let _ = env_logger::builder().try_init();
    match Command::parse() {
        Command::Mutate(params) => {
            let mut mutant_gen = MutantGenerator::new(params);
            mutant_gen.run();
        }
    }
}

// TODO: add the case where we have specific functions from the user to mutate.
// TODO: why aren't we generating enough mutants?
// TODO: why one same mutant: because the original file didn't have spaces a**10, and the tool fails to recognize the difference between a ** 10 and a**10.
