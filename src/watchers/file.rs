use ansi_to_tui::IntoText;
use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, Watcher as _};
use std::{
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::Arc,
};

use tokio::sync::{mpsc::UnboundedSender, Mutex};

use crate::circular::CircularBuffer;

use super::Watcher;

pub struct FileWatcher {
    pub path: String,
    pub history: CircularBuffer<tui::text::Spans<'static>>,
    pub pos: u64,
    handle: Option<RecommendedWatcher>,
}

impl FileWatcher {
    pub fn new(file: &str) -> anyhow::Result<Arc<Mutex<Self>>> {
        Ok(Arc::new(Mutex::new(Self {
            path: file.into(),
            history: CircularBuffer::new(10000),
            pos: 0,
            handle: None,
        })))
    }

    pub fn iter_tail<'b>(&'b self, n: usize) -> impl Iterator<Item = &tui::text::Spans<'b>> {
        self.history
            .iter()
            .skip(self.history.len().saturating_sub(n))
    }
}

impl Watcher for FileWatcher {
    fn start(&mut self, tx: UnboundedSender<()>) -> anyhow::Result<()> {
        // trigger a first read on startup
        tx.send(()).unwrap();

        let path = self.path.clone();
        let mut watcher = recommended_watcher(move |res| match res {
            // only notify for Modify events
            Ok(Event {
                kind: EventKind::Modify(..),
                ..
            }) => tx.send(()).unwrap(),
            Ok(_) => {}

            Err(e) => println!("error: {:?}", e),
        })?;
        watcher.watch(Path::new(&path), notify::RecursiveMode::NonRecursive)?;

        self.handle = Some(watcher);

        Ok(())
    }

    fn poll(&mut self) {
        let mut f = std::fs::File::open(&self.path).unwrap();
        let new_len = f.metadata().unwrap().len();

        // read new contents
        f.seek(SeekFrom::Start(self.pos)).unwrap();
        let mut new_contents = vec![];
        f.read_to_end(&mut new_contents).unwrap();
        // f.read_to_string(&mut new_contents).unwrap();
        self.pos = new_len;

        // push each new line to history
        for line in new_contents.into_text().unwrap().lines.iter() {
            self.history.push(line.clone());
        }
    }
}
