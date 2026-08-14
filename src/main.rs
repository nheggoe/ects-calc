mod calculator;
mod model;
mod persistence;

use model::{Semester, Subject};
use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};
use std::path::PathBuf;

struct App {
    semesters: Vec<Semester>,
    path: PathBuf,
    selected: usize,
    mode: Mode,
}

enum Mode {
    Browsing,
    Editing(Form),
}

struct Form {
    field: Field,
    name: String,
    credit: String,
    semester: String,
    grade: String,
    editing: Option<usize>,
    error: Option<String>,
}

impl Form {
    fn new_add(default_semester: usize) -> Form {
        Form {
            field: Field::Name,
            name: String::new(),
            credit: String::new(),
            semester: default_semester.to_string(),
            grade: String::new(),
            editing: None,
            error: None,
        }
    }

    fn new_edit(flat_index: usize, semester: usize, subject: &Subject) -> Form {
        Form {
            field: Field::Name,
            name: subject.name.clone(),
            credit: subject.credit.to_string(),
            semester: semester.to_string(),
            grade: persistence::format_grade(&subject.result),
            editing: Some(flat_index),
            error: None,
        }
    }

    fn field_mut(&mut self) -> &mut String {
        match self.field {
            Field::Name => &mut self.name,
            Field::Credit => &mut self.credit,
            Field::Semester => &mut self.semester,
            Field::Grade => &mut self.grade,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Name,
    Credit,
    Semester,
    Grade,
}

impl Field {
    fn next(self) -> Field {
        match self {
            Field::Name => Field::Credit,
            Field::Credit => Field::Semester,
            Field::Semester => Field::Grade,
            Field::Grade => Field::Name,
        }
    }

    fn prev(self) -> Field {
        match self {
            Field::Name => Field::Grade,
            Field::Credit => Field::Name,
            Field::Semester => Field::Credit,
            Field::Grade => Field::Semester,
        }
    }
}

fn main() -> std::io::Result<()> {
    let path = persistence::default_path().expect("could not resolve data file path");
    let mut semesters = match persistence::load(&path) {
        Ok(semesters) => semesters,
        Err(persistence::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => panic!("failed to load {}: {e}", path.display()),
    };
    semesters.sort_by_key(|s| s.number);

    let mut app = App {
        semesters,
        path,
        selected: 0,
        mode: Mode::Browsing,
    };

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app))?;
        if handle_event(app)? {
            return Ok(());
        }
    }
}

fn handle_event(app: &mut App) -> std::io::Result<bool> {
    let Event::Key(key) = event::read()? else {
        return Ok(false);
    };
    if key.kind != KeyEventKind::Press {
        return Ok(false);
    }

    let quit = match app.mode {
        Mode::Browsing => handle_browsing_key(app, key.code),
        Mode::Editing(_) => {
            handle_editing_key(app, key.code);
            false
        }
    };
    Ok(quit)
}

fn flatten(semesters: &[Semester]) -> Vec<(usize, usize)> {
    semesters
        .iter()
        .enumerate()
        .flat_map(|(si, semester)| (0..semester.subjects.len()).map(move |ui| (si, ui)))
        .collect()
}

fn default_semester(semesters: &[Semester]) -> usize {
    semesters.iter().map(|s| s.number).max().unwrap_or(1)
}

fn handle_browsing_key(app: &mut App, code: KeyCode) -> bool {
    let flat = flatten(&app.semesters);
    match code {
        KeyCode::Char('q') | KeyCode::Esc => return true,
        KeyCode::Down | KeyCode::Char('j') => {
            if !flat.is_empty() {
                app.selected = (app.selected + 1).min(flat.len() - 1);
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.selected = app.selected.saturating_sub(1);
        }
        KeyCode::Char('a') => {
            app.mode = Mode::Editing(Form::new_add(default_semester(&app.semesters)));
        }
        KeyCode::Char('e') => {
            if let Some(&(si, ui)) = flat.get(app.selected) {
                let subject = &app.semesters[si].subjects[ui];
                app.mode = Mode::Editing(Form::new_edit(
                    app.selected,
                    app.semesters[si].number,
                    subject,
                ));
            }
        }
        KeyCode::Char('d') | KeyCode::Char('x') => {
            if let Some(&(si, ui)) = flat.get(app.selected) {
                remove_subject(&mut app.semesters, si, ui);
                let remaining = flatten(&app.semesters).len();
                if app.selected > 0 && app.selected >= remaining {
                    app.selected -= 1;
                }
                persistence::save(&app.path, &app.semesters).expect("failed to save subjects");
            }
        }
        _ => {}
    }
    false
}

fn handle_editing_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Browsing;
            return;
        }
        KeyCode::Enter => {
            submit_form(app);
            return;
        }
        _ => {}
    }

    let Mode::Editing(form) = &mut app.mode else {
        return;
    };
    match code {
        KeyCode::Tab | KeyCode::Down => form.field = form.field.next(),
        KeyCode::BackTab | KeyCode::Up => form.field = form.field.prev(),
        KeyCode::Backspace => {
            form.field_mut().pop();
        }
        KeyCode::Char(c) => {
            form.field_mut().push(c);
        }
        _ => {}
    }
}

fn submit_form(app: &mut App) {
    let Mode::Editing(form) = &app.mode else {
        return;
    };
    let name = form.name.trim().to_string();
    let credit_input = form.credit.trim().to_string();
    let semester_input = form.semester.trim().to_string();
    let grade_input = form.grade.trim().to_string();
    let editing = form.editing;

    if name.is_empty() {
        set_form_error(app, "name cannot be empty");
        return;
    }
    let credit: f64 = match credit_input.parse() {
        Ok(v) => v,
        Err(_) => {
            set_form_error(app, "credit must be a number");
            return;
        }
    };
    let semester: usize = match semester_input.parse() {
        Ok(v) => v,
        Err(_) => {
            set_form_error(app, "semester must be a whole number");
            return;
        }
    };
    let result = match persistence::parse_grade(&grade_input) {
        Ok(o) => o,
        Err(_) => {
            set_form_error(app, "grade must be A-E, Pass, or Fail");
            return;
        }
    };

    if let Some(flat_index) = editing
        && let Some(&(si, ui)) = flatten(&app.semesters).get(flat_index)
    {
        remove_subject(&mut app.semesters, si, ui);
    }
    insert_subject(
        &mut app.semesters,
        semester,
        Subject {
            name,
            credit,
            result,
        },
    );
    app.semesters.sort_by_key(|s| s.number);
    persistence::save(&app.path, &app.semesters).expect("failed to save subjects");
    app.mode = Mode::Browsing;
}

fn set_form_error(app: &mut App, message: &str) {
    if let Mode::Editing(form) = &mut app.mode {
        form.error = Some(message.to_string());
    }
}

fn remove_subject(semesters: &mut Vec<Semester>, semester_index: usize, subject_index: usize) {
    semesters[semester_index].subjects.remove(subject_index);
    if semesters[semester_index].subjects.is_empty() {
        semesters.remove(semester_index);
    }
}

fn insert_subject(semesters: &mut Vec<Semester>, number: usize, subject: Subject) {
    if let Some(semester) = semesters.iter_mut().find(|s| s.number == number) {
        semester.subjects.push(subject);
    } else {
        semesters.push(Semester {
            number,
            subjects: vec![subject],
        });
    }
}

fn render(frame: &mut Frame, app: &App) {
    let [list_area, stats_area, hint_area] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let mut items = Vec::new();
    let mut idx = 0;
    for semester in &app.semesters {
        items.push(ListItem::new(
            Line::from(format!("Semester {}", semester.number)).bold(),
        ));
        for subject in &semester.subjects {
            let selected = idx == app.selected;
            let marker = if selected { "> " } else { "  " };
            let text = format!(
                "{marker}{:<20} {:>5.1} ECTS  {}",
                subject.name,
                subject.credit,
                persistence::format_grade(&subject.result)
            );
            let mut line = Line::from(text);
            if selected {
                line = line.reversed();
            }
            items.push(ListItem::new(line));
            idx += 1;
        }
    }

    frame.render_widget(
        List::new(items).block(Block::bordered().title("ects-calc")),
        list_area,
    );

    let average = calculator::overall_average(&app.semesters);
    let valid = calculator::valid_credits(&app.semesters);
    frame.render_widget(
        Paragraph::new(format!(
            "Average: {average:.2}   Valid credits: {valid:.1} ECTS"
        )),
        stats_area,
    );
    frame.render_widget(
        Paragraph::new("a: add   e: edit   d: delete   q: quit"),
        hint_area,
    );

    if let Mode::Editing(form) = &app.mode {
        let title = if form.editing.is_some() {
            "Edit Subject"
        } else {
            "Add Subject"
        };
        render_form(frame, form, title);
    }
}

fn render_form(frame: &mut Frame, form: &Form, title: &str) {
    let area = centered_rect(50, 40, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::bordered().title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);

    frame.render_widget(
        field_line("Name", &form.name, form.field == Field::Name),
        rows[0],
    );
    frame.render_widget(
        field_line("Credit", &form.credit, form.field == Field::Credit),
        rows[1],
    );
    frame.render_widget(
        field_line("Semester", &form.semester, form.field == Field::Semester),
        rows[2],
    );
    frame.render_widget(
        field_line("Grade", &form.grade, form.field == Field::Grade),
        rows[3],
    );
    frame.render_widget(
        Paragraph::new(form.error.clone().unwrap_or_default()).red(),
        rows[4],
    );
    frame.render_widget(
        Paragraph::new("Tab: next field   Enter: save   Esc: cancel"),
        rows[5],
    );
}

fn field_line(label: &str, value: &str, focused: bool) -> Paragraph<'static> {
    let text = format!("{label:<9}{value}");
    let paragraph = Paragraph::new(text);
    if focused {
        paragraph.reversed()
    } else {
        paragraph
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}
