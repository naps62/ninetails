mod args;
mod ui;

fn main() -> Result<(), std::io::Error> {
    let args = args::parse();

    ui::run(args)?;

    Ok(())
}
