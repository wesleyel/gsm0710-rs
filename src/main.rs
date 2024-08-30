use clap::Parser;
use cli::Args;

mod cli;

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}
