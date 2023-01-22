use std::process;

use redisbrute::Args;
use clap::{Parser};

fn main() {
    let args: Args = Args::parse();
    use std::time::Instant;
    let now = Instant::now();
    if let Err(e) = redisbrute::run(args) {
        println!("Application error: {e}");
        process::exit(1);
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

