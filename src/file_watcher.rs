use std::{
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::Path,
    sync::Arc,
};

use tokio::sync::{
    mpsc::{self, Sender},
    Mutex,
};

use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, Watcher};

use crate::circular::CircularBuffer;

pub struct FileWatcher {
    path: String,
    pub history: CircularBuffer<String>,
}

impl FileWatcher {
    pub fn new(file: &str) -> anyhow::Result<Arc<Mutex<Self>>> {
        Ok(Arc::new(Mutex::new(Self {
            path: file.into(),
            history: CircularBuffer::new(10000),
        })))
    }

    pub fn iter_tail<'a>(&'a self, n: usize) -> impl Iterator<Item = &String> {
        self.history
            .iter()
            .skip(self.history.len().saturating_sub(n))
    }
}

pub async fn listen(
    obj: &Arc<Mutex<FileWatcher>>,
    outer_tx: Sender<()>,
) -> anyhow::Result<RecommendedWatcher> {
    // channel for internal comms between the new thread and the file watcher
    let (inner_tx, mut inner_rx) = mpsc::unbounded_channel();

    // setup a task to listen to file changes and read new lines
    let copy = obj.clone();
    tokio::task::spawn(async move {
        let mut pos = 0;
        let mut new_contents = String::new();
        loop {
            match inner_rx.recv().await {
                // file was modified
                Some(_) => {
                    let mut watcher = copy.lock().await;
                    let mut f = std::fs::File::open(&watcher.path).unwrap();
                    let new_len = f.metadata().unwrap().len();

                    // dbg!(new_len);
                    // read new contents
                    f.seek(SeekFrom::Start(pos)).unwrap();
                    new_contents.clear();
                    f.read_to_string(&mut new_contents).unwrap();
                    pos = new_len;

                    // push each new line to history
                    for line in new_contents.lines() {
                        watcher.history.push(line.into());
                    }

                    // ping the outer channel to trigger a re-render
                    outer_tx.send(()).await.unwrap();
                }
                _ => {}
            }
        }
    });

    // trigger a read on startup
    inner_tx.send(()).unwrap();

    // start watching
    let mut watcher = recommended_watcher(move |res| match res {
        // only notify for Modify events
        Ok(Event {
            kind: EventKind::Modify(..),
            ..
        }) => inner_tx.send(()).unwrap(),
        Ok(_) => {}

        Err(e) => println!("error: {:?}", e),
    })?;
    let path = obj.lock().await.path.clone();
    watcher.watch(Path::new(&path), notify::RecursiveMode::NonRecursive)?;

    Ok(watcher)
}
