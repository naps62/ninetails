use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    pub file1: String,
    #[arg(long)]
    pub file2: String,
}

pub fn parse() -> Args {
    Args::parse()
}
