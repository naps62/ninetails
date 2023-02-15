use std::{
    collections::VecDeque,
    io::{BufRead, BufReader},
    iter,
    path::Path,
    slice::Iter,
    sync::{Arc, Mutex},
    thread,
};

use tokio::sync::mpsc::{self, Receiver, Sender, UnboundedReceiver};

use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, Watcher};

use crate::circular::CircularBuffer;

pub struct FileWatcher {
    path: String,
    pub history: CircularBuffer<String>,
    following: bool,
}

impl FileWatcher {
    pub fn new(file: &str) -> anyhow::Result<Arc<Mutex<Self>>> {
        Ok(Arc::new(Mutex::new(Self {
            path: file.into(),
            history: CircularBuffer::new(10000),
            following: true,
        })))
    }

    pub fn iter_tail<'a>(&'a self, n: usize) -> impl Iterator<Item = &String> {
        self.history
            .iter()
            .skip(self.history.len().saturating_sub(n))
    }
}

pub fn listen(
    obj: &Arc<Mutex<FileWatcher>>,
) -> anyhow::Result<(RecommendedWatcher, UnboundedReceiver<()>)> {
    // channel for notifying a consumer that updates happened
    let (outer_tx, outer_rx) = mpsc::unbounded_channel();

    // channel for internal comms between the new thread and the file watcher
    let (inner_tx, mut inner_rx) = mpsc::unbounded_channel();

    let mut watcher = recommended_watcher(move |res| match res {
        // only notify for Modify events
        Ok(Event {
            kind: EventKind::Modify(..),
            ..
        }) => inner_tx.send(()).unwrap(),
        Ok(_) => {}

        Err(e) => println!("error: {:?}", e),
    })?;

    watcher.watch(Path::new("test.log"), notify::RecursiveMode::NonRecursive)?;

    let copy = obj.clone();
    tokio::task::spawn(async move {
        loop {
            match inner_rx.recv().await {
                // file was modified
                Some(_) => {
                    let mut watcher = copy.lock().unwrap();
                    let f = std::fs::File::open(&watcher.path).unwrap();
                    let reader = BufReader::new(f);

                    // TODO: this needs to be the size of the window
                    let mut iter = reader.lines();
                    while let Some(Ok(line)) = iter.next() {
                        watcher.history.push(line);
                    }

                    // ping the outer channel to trigger a re-render
                    outer_tx.send(()).unwrap();
                }
                _ => {}
            }
        }
    });

    Ok((watcher, outer_rx))
}
