use std::sync::Arc;

use notify::INotifyWatcher;
use tokio::sync::{mpsc::Receiver, Mutex};

use crate::{
    args::Args,
    file_watcher::{self, FileWatcher},
};

pub struct App {
    pub receiver: Receiver<()>,
    pub watch_handles: Vec<INotifyWatcher>,
    pub watchers: Vec<Arc<Mutex<FileWatcher>>>,
}

impl App {
    pub async fn new(args: Args) -> anyhow::Result<Self> {
        let mut watch_handles: Vec<_> = vec![];
        let mut watchers: Vec<_> = vec![];

        let (tx, rx) = tokio::sync::mpsc::channel::<()>(100);

        let watcher = FileWatcher::new(&args.file1)?;
        let handle = file_watcher::listen(&watcher, tx.clone()).await?;
        watch_handles.push(handle);
        watchers.push(watcher);

        let watcher2 = FileWatcher::new(&args.file2)?;
        let handle = file_watcher::listen(&watcher2, tx.clone()).await?;
        watch_handles.push(handle);
        watchers.push(watcher2);

        Ok(Self {
            watch_handles,
            watchers,
            receiver: rx,
        })
    }

    pub async fn wait(&mut self) {
        self.receiver.recv().await;
    }
}
