use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub files: Vec<String>,
}

pub fn parse() -> Args {
    Args::parse()
}
