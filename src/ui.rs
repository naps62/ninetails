use crate::args::Args;
use notify::recommended_watcher;
use notify::Watcher;
use std::io;
use std::io::BufRead;
use std::sync::mpsc;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

pub fn run(args: Args) -> Result<(), io::Error> {
    println!("{:?}", args);

    //
    // setup
    //
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run_app(&mut terminal)?;

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let path = "test.log";
    let (tx, rx) = mpsc::channel();
    let mut watcher = recommended_watcher(tx).unwrap();
    watcher
        .watch(
            std::path::Path::new(path),
            notify::RecursiveMode::NonRecursive,
        )
        .unwrap();

    let mut contents: Vec<String> = vec![];

    loop {
        terminal.draw(|f| ui(f, &contents))?;

        if let Event::Key(key) = event::read()? {
            if let KeyCode::Char('q') = key.code {
                return Ok(());
            }
        }

        // TODO: this needs to be threaded
        match rx.recv() {
            Ok(_) => {
                let f = std::fs::File::open(&path).unwrap();
                let reader = rev_buf_reader::RevBufReader::new(f);
                contents.clear();

                contents.clear();
                // TODO: this needs to be the size of the window
                contents = reader.lines().take(5).map(|l| l.unwrap()).collect();
            }

            Err(err) => {
                eprintln!("Error: {:?}", err);
                std::process::exit(1);
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, contents: &[String]) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let text: Vec<_> = contents
        .iter()
        .rev()
        .map(|l| Spans::from(vec![Span::raw(l)]))
        .collect();
    let block = Paragraph::new(text)
        .block(Block::default().title("Paragraph").borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: true });
    f.render_widget(block, chunks[0]);

    let block = Block::default().title("Block #2").borders(Borders::ALL);
    f.render_widget(block, chunks[1]);
}
