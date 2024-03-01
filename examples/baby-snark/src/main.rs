use std::fs::File;
use std::{io, ops::Neg};

use baby_snark::ProvingKey;
use baby_snark::{
    common::FrElement, scs::SquareConstraintSystem, setup, ssp::SquareSpanProgram, verify, Prover,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Solution {
    u: Vec<Vec<i64>>,
    public: Vec<i64>,
    witness: Vec<i64>,
}
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'f', long = "file", required=false)]
    matrix: String,
    #[arg(long = "pkf", required=false, default_value_t = String::from("placeholder"))]
    proving_file: String,
    #[arg(long = "vkf", required=false, default_value_t = String::from("placeholder"))]
    verifying_file: String,
    #[arg(long = "proof", required=false, default_value_t = String::from("placeholder"))]
    proof: String,
    #[arg(long = "setup", default_value_t = false)]
    setup: bool,
    #[arg(long = "prover", default_value_t = false)]
    prover: bool,
    #[arg(long = "verifier", default_value_t = false)]
    verifier: bool,
    #[arg(long = "all", default_value_t = false)]
    all: bool,
}

fn main() {
    let args = Args::parse();
    let program = File::open(args.matrix).unwrap();
    let reader = io::BufReader::new(program);

    let sol: Solution = serde_json::from_reader(reader).unwrap();
    let u = i64_matrix_to_field(sol.u);
    let public = i64_vec_to_field(sol.public);
    let witness = i64_vec_to_field(sol.witness);

    let mut input = public.clone();
    input.extend(witness.clone());
    let ssp = SquareSpanProgram::from_scs(SquareConstraintSystem::from_matrix(u.clone(), public.len()));

    match args {
        Args { setup: true, .. } => {
            let (proving_key, verifying_key) = setup(&ssp);
            todo!("serealize ssp, pk, vk");
        }
        Args { prover: true, .. } => {
            let pk = File::open(args.proving_file).unwrap();
            let reader = io::BufReader::new(pk);

            //let proof = Prover::prove(&input, &ssp, &proving_key);
            todo!("deserealize proving_key and serealize proof");
        }
        Args { verifier: true, .. } => {
            let proof = File::open(args.proof).unwrap();
            let reader_proof = io::BufReader::new(proof);

            let vk = File::open(args.verifying_file).unwrap();
            let reader_vk = io::BufReader::new(vk);

            //let verify(&verifying_key, &proof, &public);
            todo!("deserealize verifying file and proof");
        }
        Args { all: true, .. } => {
            let test = test_integration(u, witness, public);

            println!("test: {test}");
        }
        _ => {
            println!("Try again");
        }
    };
}

fn test_integration(
    u: Vec<Vec<FrElement>>,
    witness: Vec<FrElement>,
    public: Vec<FrElement>,
) -> bool {
    let mut input = public.clone();
    input.extend(witness.clone());

    let ssp = SquareSpanProgram::from_scs(SquareConstraintSystem::from_matrix(u, public.len()));
    let (proving_key, verifying_key) = setup(&ssp);

    let proof = Prover::prove(&input, &ssp, &proving_key);
    verify(&verifying_key, &proof, &public)
}

fn i64_to_field(element: &i64) -> FrElement {
    let mut fr_element = FrElement::from(element.unsigned_abs());
    if element.is_negative() {
        fr_element = fr_element.neg()
    }

    fr_element
}

fn i64_vec_to_field(elements: Vec<i64>) -> Vec<FrElement> {
    elements.iter().map(i64_to_field).collect()
}

fn i64_matrix_to_field(elements: Vec<Vec<i64>>) -> Vec<Vec<FrElement>> {
    let mut matrix = Vec::new();
    for f in elements {
        matrix.push(i64_vec_to_field(f));
    }
    matrix
}
