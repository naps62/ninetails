use crate::{
    args::Args,
    file_watcher::{self, FileWatcher},
};
use std::{
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{future::FutureExt, StreamExt};
use futures_timer::Delay;
use tokio::{select, time::sleep};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

pub async fn run(args: Args) -> anyhow::Result<()> {
    //
    // setup
    //
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run_app(&mut terminal).await?;

    //
    // teardown
    //
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> anyhow::Result<()> {
    let watcher = FileWatcher::new("test.log")?;
    let (_handle, mut updates) = file_watcher::listen(&watcher)?;
    let mut term_events = EventStream::new();

    'mainloop: loop {
        terminal.draw(|f| ui(f, &watcher))?;

        select! {
            Some(_) = updates.recv() =>{
                /* update was triggered. looping */
            }
            maybe_event = term_events.next() => {
                match maybe_event {
                    Some(Ok(event))=>{

                        // q pressed, quit
                        if event== Event::Key(KeyCode::Char('q').into()){
                            break 'mainloop;
                        }
                    }
                    Some(Err(e))=>{println!("Error: {:?}", e)},
                    None=>break
                }
            }
        }
    }

    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, watcher: &Arc<Mutex<FileWatcher>>) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    // parse current view into a block
    let guard = watcher.lock().unwrap();
    let text: Vec<_> = guard
        .iter_tail(chunks[0].height as usize - 2)
        .map(|l| Spans::from(vec![Span::raw(l)]))
        .collect();

    let block = Paragraph::new(text)
        .block(Block::default().title("File 1").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: true });
    f.render_widget(block, chunks[0]);

    let block = Block::default().title("Block #2").borders(Borders::ALL);
    f.render_widget(block, chunks[1]);
}
