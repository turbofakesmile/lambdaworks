use std::fmt::format;
use std::fs::File;
use std::{io, ops::Neg};
use std::io::{Read, Write};

use baby_snark::{ProvingKey, VerifyingKey};
use baby_snark::{
    common::FrElement, scs::SquareConstraintSystem, setup, ssp::SquareSpanProgram, verify, Prover, Proof,
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
    let program = File::open(args.matrix.clone()).unwrap();
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
            
            //TODO! serialize pk, vk

            //let filename = args.matrix.split('.').next().unwrap();
            //let mut data_file = File::create(format!("{}.pk", filename)).expect("pk file creation failed");
            //data_file.write(&proving_key.serialize()).expect("pk write failed");
            //data_file.close()
            //data_file = File::create(format!("{}.vk", filename)).expect("vk file creation failed");
            //data_file.write(&verifying_key.serialize()).expect("vk write failed");
        }
        Args { prover: true, .. } => {
            let mut pk_file = File::open(args.proving_file).unwrap();

            let mut pk = Vec::new();
            pk_file.read_to_end(&mut pk).unwrap();
            
            //let verifying_key = VerifyingKey::deserialize(&vk).unwrap();

            //TODO! serialize proof

            //let proof = Prover::prove(&input, &ssp, &proving_key);
            //let filename = args.matrix.split('.').next().unwrap();
            //let mut data_file = File::create(format!("{}.proof", filename)).expect("creation failed");
            //data_file.write(&proof.serialize).expect("write failed");
        
            todo!("deserealize proving_key");
        }
        Args { verifier: true, .. } => {
            let mut proof_file = File::open(args.proof).unwrap();
            let mut vk_file = File::open(args.verifying_file).unwrap();

            let mut proof = Vec::new();
            proof_file.read_to_end(&mut proof).unwrap();
            let proof = Proof::deserialize(&proof).unwrap();

            let mut vk = Vec::new();
            vk_file.read_to_end(&mut vk).unwrap();
            //let verifying_key = VerifyingKey::deserialize(&vk).unwrap();

            //let output = verify(&verifying_key, &proof, &public);
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
