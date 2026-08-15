use crate::calculator;
use crate::model::{Outcome, Semester, Subject};
use crate::persistence;
use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};

pub struct App {
    pub semesters: Vec<Semester>,
    pub selected: usize,
    pub mode: Mode,
}

impl App {
    pub fn new(mut semesters: Vec<Semester>) -> App {
        semesters.sort_by_key(|s| s.number);
        App {
            semesters,
            selected: 0,
            mode: Mode::Browsing,
        }
    }
}

pub enum Mode {
    Browsing,
    Editing(Form),
}

pub struct Form {
    field: Field,
    code: String,
    name: String,
    credit: String,
    semester: String,
    grade: String,
    potential: String,
    editing: Option<usize>,
    error: Option<String>,
}

impl Form {
    fn new_add(default_semester: usize) -> Form {
        Form {
            field: Field::Code,
            code: String::new(),
            name: String::new(),
            credit: "7.5".to_string(),
            semester: default_semester.to_string(),
            grade: String::new(),
            potential: String::new(),
            editing: None,
            error: None,
        }
    }

    fn new_edit(flat_index: usize, semester: usize, subject: &Subject) -> Form {
        Form {
            field: Field::Code,
            code: subject.code.to_uppercase(),
            name: subject.name.clone(),
            credit: subject.credit.to_string(),
            semester: semester.to_string(),
            grade: persistence::format_grade(&subject.result).to_uppercase(),
            potential: subject
                .potential
                .map(persistence::grade_letter)
                .unwrap_or_default()
                .to_string(),
            editing: Some(flat_index),
            error: None,
        }
    }

    fn field_mut(&mut self) -> &mut String {
        match self.field {
            Field::Code => &mut self.code,
            Field::Name => &mut self.name,
            Field::Credit => &mut self.credit,
            Field::Semester => &mut self.semester,
            Field::Grade => &mut self.grade,
            Field::Potential => &mut self.potential,
        }
    }

    /// Appends a typed character, uppercasing it first for fields that are
    /// always displayed in caps (subject code, grade, potential grade).
    fn push_char(&mut self, c: char) {
        let c = if matches!(self.field, Field::Code | Field::Grade | Field::Potential) {
            c.to_ascii_uppercase()
        } else {
            c
        };
        self.field_mut().push(c);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Code,
    Name,
    Credit,
    Semester,
    Grade,
    Potential,
}

impl Field {
    fn next(self) -> Field {
        match self {
            Field::Code => Field::Name,
            Field::Name => Field::Grade,
            Field::Grade => Field::Potential,
            Field::Potential => Field::Semester,
            Field::Semester => Field::Credit,
            Field::Credit => Field::Code,
        }
    }

    fn prev(self) -> Field {
        match self {
            Field::Code => Field::Credit,
            Field::Name => Field::Code,
            Field::Grade => Field::Name,
            Field::Potential => Field::Grade,
            Field::Semester => Field::Potential,
            Field::Credit => Field::Semester,
        }
    }
}

/// What a key press did to the app, so the caller can decide whether/how to persist.
/// `app.rs` has no opinion on storage (file, ssh session, browser localStorage, ...) —
/// that's entirely up to whoever calls `handle_key`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Response {
    None,
    Changed,
    Quit,
}

/// Dispatches a key press.
pub fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Response {
    match app.mode {
        Mode::Browsing => handle_browsing_key(app, code, modifiers),
        Mode::Editing(_) => handle_editing_key(app, code),
    }
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

fn handle_browsing_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Response {
    let flat = flatten(&app.semesters);
    match code {
        KeyCode::Char('q') | KeyCode::Esc => return Response::Quit,
        KeyCode::Up if modifiers.contains(KeyModifiers::SHIFT) => {
            return move_subject(app, -1);
        }
        KeyCode::Down if modifiers.contains(KeyModifiers::SHIFT) => {
            return move_subject(app, 1);
        }
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
                return Response::Changed;
            }
        }
        KeyCode::Char('t') => {
            if let Some(&(si, ui)) = flat.get(app.selected) {
                let subject = &mut app.semesters[si].subjects[ui];
                subject.included = !subject.included;
                return Response::Changed;
            }
        }
        _ => {}
    }
    Response::None
}

fn handle_editing_key(app: &mut App, code: KeyCode) -> Response {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Browsing;
            return Response::None;
        }
        KeyCode::Enter => {
            return submit_form(app);
        }
        _ => {}
    }

    let Mode::Editing(form) = &mut app.mode else {
        return Response::None;
    };
    match code {
        KeyCode::Tab | KeyCode::Down => form.field = form.field.next(),
        KeyCode::BackTab | KeyCode::Up => form.field = form.field.prev(),
        KeyCode::Backspace => {
            form.field_mut().pop();
        }
        KeyCode::Char(c) => {
            form.push_char(c);
        }
        _ => {}
    }
    Response::None
}

fn submit_form(app: &mut App) -> Response {
    let Mode::Editing(form) = &app.mode else {
        return Response::None;
    };
    let code = form.code.trim().to_string();
    let name = form.name.trim().to_string();
    let credit_input = form.credit.trim().to_string();
    let semester_input = form.semester.trim().to_string();
    let grade_input = form.grade.trim().to_string();
    let potential_input = form.potential.trim().to_string();
    let editing = form.editing;

    let credit: f64 = match credit_input.parse() {
        Ok(v) => v,
        Err(_) => {
            set_form_error(app, "credit must be a number");
            return Response::None;
        }
    };
    let semester: usize = match semester_input.parse() {
        Ok(v) => v,
        Err(_) => {
            set_form_error(app, "semester must be a whole number");
            return Response::None;
        }
    };
    let result = match persistence::parse_grade(&grade_input) {
        Ok(o) => o,
        Err(_) => {
            set_form_error(app, "grade must be A-E, Pass, or Fail");
            return Response::None;
        }
    };
    let potential = if potential_input.is_empty() {
        None
    } else {
        match persistence::parse_grade_letter(&potential_input) {
            Ok(g) => Some(g),
            Err(_) => {
                set_form_error(app, "potential grade must be A-E");
                return Response::None;
            }
        }
    };

    let mut included = true;
    let mut old_location = None;
    if let Some(flat_index) = editing
        && let Some(&(si, ui)) = flatten(&app.semesters).get(flat_index)
    {
        included = app.semesters[si].subjects[ui].included;
        old_location = Some((app.semesters[si].number, si, ui));
    }

    let subject = Subject {
        code,
        name,
        credit,
        result,
        included,
        potential,
    };

    match old_location {
        // Editing without changing semester: replace in place so the
        // subject keeps its position instead of jumping to the end.
        Some((old_semester, si, ui)) if old_semester == semester => {
            app.semesters[si].subjects[ui] = subject;
        }
        // Moved to a different semester: it has no natural position there,
        // so append it and move the selection along with it.
        Some((_, si, ui)) => {
            remove_subject(&mut app.semesters, si, ui);
            insert_subject(&mut app.semesters, semester, subject);
            app.semesters.sort_by_key(|s| s.number);
            if let Some(target_si) = app.semesters.iter().position(|s| s.number == semester) {
                let target_ui = app.semesters[target_si].subjects.len() - 1;
                if let Some(flat) = flatten(&app.semesters)
                    .iter()
                    .position(|&p| p == (target_si, target_ui))
                {
                    app.selected = flat;
                }
            }
        }
        None => {
            insert_subject(&mut app.semesters, semester, subject);
            app.semesters.sort_by_key(|s| s.number);
        }
    }

    app.mode = Mode::Browsing;
    Response::Changed
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

/// Swaps the selected subject with its neighbor within the same semester
/// (`direction` -1 = up, +1 = down), moving the selection along with it.
/// Does nothing at the top/bottom of a semester's list — reordering across
/// semesters happens through the edit form's semester field instead.
fn move_subject(app: &mut App, direction: isize) -> Response {
    let flat = flatten(&app.semesters);
    let Some(&(si, ui)) = flat.get(app.selected) else {
        return Response::None;
    };
    let subjects = &mut app.semesters[si].subjects;
    let new_ui = if direction < 0 {
        ui.checked_sub(1)
    } else {
        (ui + 1 < subjects.len()).then_some(ui + 1)
    };
    let Some(new_ui) = new_ui else {
        return Response::None;
    };
    subjects.swap(ui, new_ui);
    if let Some(new_flat) = flatten(&app.semesters)
        .iter()
        .position(|&p| p == (si, new_ui))
    {
        app.selected = new_flat;
    }
    Response::Changed
}

/// Truncates to at most `max_chars` characters, so fixed-width columns stay
/// aligned even when the underlying text is longer than the column.
fn truncate(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

/// Widths for the code/name columns, sized to the longest value actually
/// present instead of a guessed constant — so nothing gets truncated unless
/// it's past the sanity cap.
fn column_widths(semesters: &[Semester]) -> (usize, usize) {
    let subjects = semesters.iter().flat_map(|s| &s.subjects);
    let code_width = subjects
        .clone()
        .map(|s| s.code.chars().count())
        .max()
        .unwrap_or(0)
        .clamp(4, 15);
    let name_width = subjects
        .map(|s| s.name.chars().count())
        .max()
        .unwrap_or(0)
        .clamp(4, 40);
    (code_width, name_width)
}

pub fn render(frame: &mut Frame, app: &App) {
    let [list_area, stats_area, hint_area] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let (code_width, name_width) = column_widths(&app.semesters);
    let mut items = Vec::new();
    let mut idx = 0;
    for semester in &app.semesters {
        items.push(ListItem::new(
            Line::from(format!("Semester {}", semester.number)).bold(),
        ));
        for subject in &semester.subjects {
            let selected = idx == app.selected;
            let marker = if selected { "> " } else { "  " };
            let code = truncate(&subject.code, code_width);
            let name = truncate(&subject.name, name_width);
            let mut grade = persistence::format_grade(&subject.result);
            if let Some(potential) = subject.potential {
                grade = format!("{grade} → {}", persistence::grade_letter(potential));
            }
            // "Pass" and "F" never carry a grade point, so they never factor
            // into the average — mark that distinctly from the included
            // toggle (which is a manual, whole-row exclusion).
            let grade_span = match subject.result {
                Outcome::Passed(None) => Span::from(grade).blue(),
                Outcome::Failed => Span::from(grade).red(),
                Outcome::Passed(Some(_)) => Span::from(grade),
            };
            let failed = matches!(subject.result, Outcome::Failed);
            let label = format!("{marker}{code:<code_width$} {name:<name_width$} ");
            let credit_number = format!("{:.1}", subject.credit);
            let credit_padding = " ".repeat(5usize.saturating_sub(credit_number.chars().count()));
            let credit_text = format!("{credit_number} ECTS");
            let credit_span = if failed {
                Span::from(credit_text).red().crossed_out()
            } else {
                Span::from(credit_text)
            };
            let mut line = Line::from(vec![
                Span::from(label),
                Span::from(credit_padding),
                credit_span,
                Span::from("  "),
                grade_span,
            ]);
            if !subject.included {
                line = line.crossed_out().dim();
            }
            if selected {
                line = line.reversed();
            }
            items.push(ListItem::new(line));
            idx += 1;
        }
    }

    frame.render_widget(
        List::new(items).block(Block::bordered().title("ECTS Calculator")),
        list_area,
    );

    let average = calculator::overall_average(&app.semesters);
    let potential_average = calculator::potential_average(&app.semesters);
    let average_text = if potential_average != average {
        format!("Average: {average:.2} → {potential_average:.2}")
    } else {
        format!("Average: {average:.2}")
    };

    let valid = calculator::valid_credits(&app.semesters);
    let potential_valid = calculator::potential_valid_credits(&app.semesters);
    let valid_text = if potential_valid != valid {
        format!("Valid credits: {valid:.1} → {potential_valid:.1} ECTS")
    } else {
        format!("Valid credits: {valid:.1} ECTS")
    };

    frame.render_widget(
        Paragraph::new(format!("{average_text}   {valid_text}")),
        stats_area,
    );
    frame.render_widget(
        Paragraph::new("a: add   e: edit   d: delete   t: toggle   shift+↑↓: move   q: quit"),
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
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);

    frame.render_widget(
        field_line("Code", &form.code, form.field == Field::Code),
        rows[0],
    );
    frame.render_widget(
        field_line("Name", &form.name, form.field == Field::Name),
        rows[1],
    );
    frame.render_widget(
        field_line("Grade", &form.grade, form.field == Field::Grade),
        rows[2],
    );
    frame.render_widget(
        field_line("Potential", &form.potential, form.field == Field::Potential),
        rows[3],
    );
    frame.render_widget(
        field_line("Semester", &form.semester, form.field == Field::Semester),
        rows[4],
    );
    frame.render_widget(
        field_line("Credit", &form.credit, form.field == Field::Credit),
        rows[5],
    );
    frame.render_widget(
        Paragraph::new(form.error.clone().unwrap_or_default()).red(),
        rows[6],
    );
    frame.render_widget(
        Paragraph::new("Tab: next field   Enter: save   Esc: cancel"),
        rows[7],
    );
}

fn field_line(label: &str, value: &str, focused: bool) -> Paragraph<'static> {
    let text = format!("{label:<10}{value}");
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
