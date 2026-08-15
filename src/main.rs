mod app;
mod calculator;
mod model;
mod persistence;

use app::{App, Response};
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use std::path::Path;

fn main() -> std::io::Result<()> {
    let path = persistence::default_path().expect("could not resolve data file path");
    let semesters = match persistence::load(&path) {
        Ok(semesters) => semesters,
        Err(persistence::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => panic!("failed to load {}: {e}", path.display()),
    };
    let mut app = App::new(semesters);

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app, &path);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App, path: &Path) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| app::render(frame, app))?;
        if handle_event(app, path)? {
            return Ok(());
        }
    }
}

fn handle_event(app: &mut App, path: &Path) -> std::io::Result<bool> {
    let Event::Key(key) = event::read()? else {
        return Ok(false);
    };
    if key.kind != KeyEventKind::Press {
        return Ok(false);
    }
    match app::handle_key(app, key.code, key.modifiers) {
        Response::Quit => Ok(true),
        Response::Changed => {
            persistence::save(path, &app.semesters).expect("failed to save subjects");
            Ok(false)
        }
        Response::None => Ok(false),
    }
}
