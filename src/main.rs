use app::App;

mod app;
mod args;
mod circular;
mod file_watcher;
mod ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = args::parse();
    let mut app = App::new(args).await?;

    ui::run(app).await?;

    Ok(())
}
