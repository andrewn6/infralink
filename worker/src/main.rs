pub mod data;
pub mod models;

fn main() {
    let start = std::time::Instant::now();

    // let memory_metadata = data::mem::memory();
    let compute_metadata = data::compute::compute_usage();

    println!("Elapsed: {} seconds", start.elapsed().as_secs_f32());
    println!("{:#?}", compute_metadata);
}
