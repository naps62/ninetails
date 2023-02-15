mod args;
mod circular;
mod file_watcher;
mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = args::parse();

    ui::run(args).await?;

    Ok(())
}
