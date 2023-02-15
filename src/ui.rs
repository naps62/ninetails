use crate::{app::App, file_watcher::FileWatcher};
use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use tokio::{select, sync::MutexGuard};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};

enum UIAction {
    SwitchTabs(usize),
    Noop,
    Quit,
}

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
            // terminal.draw is not async, so we need to grab MutexGuards for all the histories
            // before
            let tails = futures::future::join_all(app.watchers.iter().map(move |w| w.lock())).await;

            terminal.draw(|f| ui(f, app, &tails))?;
        }

        // wait for events
        select! {
            () = app.wait() =>{
                /* update was triggered by one of the files. looping */
            }
            Some(maybe_event) = term_events.next() => {
                match translate_event(maybe_event) {
                    UIAction::SwitchTabs(n) => app.move_to_tab(n),
                    UIAction::Noop => {},
                    UIAction::Quit=> break 'mainloop,
                };
            }
        }
    }

    Ok(())
}

fn translate_event(event: crossterm::Result<Event>) -> UIAction {
    use UIAction::*;

    match event {
        Ok(Event::Key(KeyEvent { code, .. })) => {
            // q pressed, quit
            match code {
                KeyCode::Char(x) if x.is_numeric() => SwitchTabs(x.to_digit(10).unwrap() as usize),
                KeyCode::Char('q') => Quit,
                _ => Noop,
            }
        }
        Ok(_) => Noop,
        Err(e) => {
            println!("Error: {:?}", e);
            Noop
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App, tails: &[MutexGuard<'_, FileWatcher>]) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let titles = vec![
        Spans::from(vec![Span::raw("All")]),
        Spans::from(vec![Span::raw("File 1")]),
        Spans::from(vec![Span::raw("File 2")]),
    ];

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(app.tab)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    f.render_widget(tabs, chunks[0]);

    match app.tab {
        0 => draw_all(f, chunks[1], tails),
        n => draw_single(f, chunks[1], &tails[n - 1], format!("File {}", n)),
    };
}

fn draw_all<B: Backend>(f: &mut Frame<B>, area: Rect, tails: &[MutexGuard<'_, FileWatcher>]) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    // f.render_widget(Vg)
    for (i, tail) in tails.iter().enumerate() {
        draw_single(f, chunks[i], tail, format!("File {}", i + 1));
    }
}

fn draw_single<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    tail: &MutexGuard<'_, FileWatcher>,
    title: String,
) {
    let text: Vec<_> = tail
        .iter_tail(area.height as usize - 2)
        .map(|l| Spans::from(vec![Span::raw(l)]))
        .collect();

    let block = Paragraph::new(text)
        .block(Block::default().title(title).borders(Borders::ALL))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: true });

    f.render_widget(block, area);
}
