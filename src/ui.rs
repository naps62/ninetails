use crate::{app::App, file_watcher::FileWatcher};
use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use tokio::select;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

pub async fn run(mut app: App) -> anyhow::Result<()> {
    // setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run_app(&mut terminal, &mut app).await?;

    // teardown
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> anyhow::Result<()> {
    let mut term_events = EventStream::new();

    'mainloop: loop {
        // render
        {
            let f1 = app.watchers[0].lock().await;
            let f2 = app.watchers[1].lock().await;

            terminal.draw(|f| ui(f, &f1, &f2))?;
        }

        // wait for events
        select! {
            () = app.wait() =>{
                /* update was triggered by one of the files. looping */
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

fn ui<B: Backend>(f: &mut Frame<B>, f1: &FileWatcher, f2: &FileWatcher) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    // parse current view into a block
    let text1: Vec<_> = f1
        .iter_tail(chunks[0].height as usize - 2)
        .map(|l| Spans::from(vec![Span::raw(l)]))
        .collect();

    let text2: Vec<_> = f2
        .iter_tail(chunks[0].height as usize - 2)
        .map(|l| Spans::from(vec![Span::raw(l)]))
        .collect();

    let block = Paragraph::new(text1)
        .block(Block::default().title("File 1").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: true });
    f.render_widget(block, chunks[0]);

    let block = Paragraph::new(text2)
        .block(Block::default().title("File 2").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: true });
    f.render_widget(block, chunks[1]);
}
