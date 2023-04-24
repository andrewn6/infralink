pub mod data;
pub mod models;

use std::{env, path::PathBuf};

fn main() {
    let proto_file = "./protos/bookstore.proto";

    tonic_build::configure()
        .build_server(false)
        .out_dir("./src")
        .compile(&[proto_file], &["."])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));
    println!("cargo:rerun-if-changed={}", proto_file);
    let start = std::time::Instant::now();

    // let memory_metadata = data::mem::memory();
    let compute_metadata = data::compute::compute_usage();

    println!("cargo:rerun-if-changed={}", proto_file);
    println!("Elapsed: {} seconds", start.elapsed().as_secs_f32());
    println!("{:#?}", compute_metadata);
}