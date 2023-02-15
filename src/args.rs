use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    // #[arg(long)]
    // pub cmd1: String,
    // #[arg(long)]
    // pub cmd2: String,
}

pub fn parse() -> Args {
    Args::parse()
}
