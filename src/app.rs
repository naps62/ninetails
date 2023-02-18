use std::sync::Arc;

use notify::INotifyWatcher;
use tokio::sync::{mpsc::Receiver, Mutex};

use crate::{
    args::Args,
    watchers::{self, file::FileWatcher},
};

pub struct App {
    pub tab: usize,
    pub receiver: Receiver<()>,
    pub watch_handles: Vec<INotifyWatcher>,
    pub watchers: Vec<Arc<Mutex<FileWatcher>>>,
}

impl App {
    pub async fn new(args: Args) -> anyhow::Result<Self> {
        let mut watch_handles: Vec<_> = vec![];
        let mut watchers: Vec<_> = vec![];

        let (tx, rx) = tokio::sync::mpsc::channel::<()>(100);

        for file in args.files {
            let watcher = FileWatcher::new(&file)?;
            let handle = watchers::file::listen(&watcher, tx.clone()).await?;
            watch_handles.push(handle);
            watchers.push(watcher);
        }

        Ok(Self {
            tab: 0,
            watch_handles,
            watchers,
            receiver: rx,
        })
    }

    pub async fn wait(&mut self) {
        self.receiver.recv().await;
    }

    pub fn move_to_tab(&mut self, n: usize) {
        if n == 0 {
            self.tab = 0;
        } else if n > self.watchers.len() + 1 {
            self.tab = 0;
        } else {
            self.tab = n - 1;
        }
    }
}
