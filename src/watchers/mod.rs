use std::sync::Arc;

use tokio::sync::{
    mpsc::{self, Sender, UnboundedSender},
    Mutex,
};

pub mod file;

pub trait Watcher {
    fn start(&mut self, tx: UnboundedSender<()>) -> anyhow::Result<()>;
    fn poll(&mut self);
}

pub async fn listen<W: Watcher + Send + 'static>(
    obj: &Arc<Mutex<W>>,
    outer_tx: Sender<()>,
) -> anyhow::Result<()> {
    // channel for internal comms between the new thread and the file watcher
    let (inner_tx, mut inner_rx) = mpsc::unbounded_channel();

    // setup a task to listen to file changes and read new lines
    let copy = obj.clone();
    tokio::task::spawn(async move {
        loop {
            match inner_rx.recv().await {
                // file was modified
                Some(_) => {
                    let mut watcher = copy.lock().await;
                    watcher.poll();

                    // ping the outer channel to trigger a re-render
                    outer_tx.send(()).await.unwrap();
                }
                _ => {}
            }
        }
    });

    // start watching
    obj.lock().await.start(inner_tx)?;

    Ok(())
}
