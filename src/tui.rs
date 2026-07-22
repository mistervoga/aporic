// Terminal UI. Calls only the existing domain functions (list_entries,
// trace_entry, complete_action, list_projects) — no business logic lives
// here, only rendering and input handling.
use crate::domain::{self, Entry, EntryFilter, EntryKind, Trace};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, ExecutableCommand};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap};
use ratatui::{Frame, Terminal};
use rusqlite::Connection;
use std::io::stdout;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    All,
    Questions,
    Actions,
    ClaimsAssumptions,
    Math,
}

const TABS: [Tab; 5] = [
    Tab::All,
    Tab::Questions,
    Tab::Actions,
    Tab::ClaimsAssumptions,
    Tab::Math,
];

impl Tab {
    fn title(self) -> &'static str {
        match self {
            Tab::All => "All",
            Tab::Questions => "Questions",
            Tab::Actions => "Actions",
            Tab::ClaimsAssumptions => "Claims & Assumptions",
            Tab::Math => "Math",
        }
    }

    fn matches(self, entry: &Entry) -> bool {
        match self {
            Tab::All => true,
            Tab::Questions => entry.kind == EntryKind::Question,
            Tab::Actions => entry.kind == EntryKind::Action,
            Tab::ClaimsAssumptions => {
                matches!(entry.kind, EntryKind::Claim | EntryKind::Assumption)
            }
            Tab::Math => entry.math_kind.is_some(),
        }
    }
}

enum RightPane {
    Detail,
    Trace(Trace),
}

struct App {
    project: Option<String>,
    projects: Vec<String>,
    project_index: usize,
    tab_index: usize,
    entries: Vec<Entry>,
    list_state: ListState,
    right: RightPane,
    status: String,
    pending_complete: bool,
    quit: bool,
}

impl App {
    fn new(conn: &Connection, project: Option<&str>) -> Result<Self> {
        let projects = domain::list_projects(conn)?;
        let project = project.map(str::to_string);
        let project_index = project
            .as_deref()
            .and_then(|name| projects.iter().position(|p| p == name))
            .unwrap_or(0);
        let mut app = Self {
            project,
            projects,
            project_index,
            tab_index: 0,
            entries: Vec::new(),
            list_state: ListState::default(),
            right: RightPane::Detail,
            status: "j/k move  enter/t detail/trace  c complete  p project  ? help  q quit"
                .to_string(),
            pending_complete: false,
            quit: false,
        };
        app.reload(conn)?;
        Ok(app)
    }

    fn reload(&mut self, conn: &Connection) -> Result<()> {
        self.entries = domain::list_entries(
            conn,
            EntryFilter {
                project: self.project.as_deref(),
                ..EntryFilter::default()
            },
        )?;
        self.right = RightPane::Detail;
        let visible = self.visible_len();
        if visible == 0 {
            self.list_state.select(None);
        } else {
            let current = self.list_state.selected().unwrap_or(0).min(visible - 1);
            self.list_state.select(Some(current));
        }
        Ok(())
    }

    fn tab(&self) -> Tab {
        TABS[self.tab_index]
    }

    fn visible(&self) -> Vec<&Entry> {
        let tab = self.tab();
        self.entries
            .iter()
            .filter(|entry| tab.matches(entry))
            .collect()
    }

    fn visible_len(&self) -> usize {
        self.visible().len()
    }

    fn selected_entry(&self) -> Option<&Entry> {
        let index = self.list_state.selected()?;
        self.visible().into_iter().nth(index)
    }

    fn next(&mut self) {
        let len = self.visible_len();
        if len == 0 {
            return;
        }
        let next = match self.list_state.selected() {
            Some(i) if i + 1 < len => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.list_state.select(Some(next));
        self.right = RightPane::Detail;
    }

    fn previous(&mut self) {
        let len = self.visible_len();
        if len == 0 {
            return;
        }
        let previous = match self.list_state.selected() {
            Some(0) | None => len - 1,
            Some(i) => i - 1,
        };
        self.list_state.select(Some(previous));
        self.right = RightPane::Detail;
    }

    fn change_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % TABS.len();
        self.list_state.select(if self.visible_len() == 0 {
            None
        } else {
            Some(0)
        });
        self.right = RightPane::Detail;
    }

    fn cycle_project(&mut self, conn: &Connection) -> Result<()> {
        if self.projects.is_empty() {
            return Ok(());
        }
        self.project_index = (self.project_index + 1) % self.projects.len();
        let name = &self.projects[self.project_index];
        self.project = (name != "global").then(|| name.clone());
        self.tab_index = 0;
        self.reload(conn)?;
        Ok(())
    }

    fn show_trace(&mut self, conn: &Connection) -> Result<()> {
        if let Some(entry) = self.selected_entry() {
            let trace = domain::trace_entry(conn, &entry.id, 2)?;
            self.right = RightPane::Trace(trace);
        }
        Ok(())
    }

    fn complete_selected(&mut self, conn: &mut Connection) -> Result<()> {
        let Some(entry) = self.selected_entry() else {
            return Ok(());
        };
        if entry.kind != EntryKind::Action {
            self.status = "only actions can be completed".to_string();
            return Ok(());
        }
        let id = entry.id.clone();
        domain::complete_action(conn, &id)?;
        self.status = "action completed".to_string();
        self.reload(conn)?;
        Ok(())
    }
}

pub fn run(conn: &mut Connection, project: Option<&str>) -> Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    out.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal, conn, project);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    conn: &mut Connection,
    project: Option<&str>,
) -> Result<()> {
    let mut app = App::new(conn, project)?;

    while !app.quit {
        terminal.draw(|frame| draw(frame, &mut app))?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if app.pending_complete {
            app.pending_complete = false;
            if key.code == KeyCode::Char('y') {
                app.complete_selected(conn)?;
            } else {
                app.status = "cancelled".to_string();
            }
            continue;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
            KeyCode::Char('j') | KeyCode::Down => app.next(),
            KeyCode::Char('k') | KeyCode::Up => app.previous(),
            KeyCode::Tab => app.change_tab(),
            KeyCode::Enter => app.right = RightPane::Detail,
            KeyCode::Char('t') => app.show_trace(conn)?,
            KeyCode::Char('p') => app.cycle_project(conn)?,
            KeyCode::Char('c') => {
                if let Some(entry) = app.selected_entry() {
                    if entry.kind == EntryKind::Action {
                        app.pending_complete = true;
                        app.status = "complete this action? (y/n)".to_string();
                    } else {
                        app.status = "only actions can be completed".to_string();
                    }
                }
            }
            KeyCode::Char('?') => {
                app.status = "full walkthrough: run  aporic tutor  outside the TUI".to_string();
            }
            _ => {}
        }
    }
    Ok(())
}

fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(area);

    draw_tabs(frame, app, rows[0]);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(rows[1]);

    draw_list(frame, app, columns[0]);
    draw_right(frame, app, columns[1]);
    draw_status(frame, app, rows[2]);
}

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = TABS.iter().map(|tab| Line::from(tab.title())).collect();
    let project_label = app.project.as_deref().unwrap_or("global");
    let tabs = Tabs::new(titles)
        .select(app.tab_index)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" aporic \u{2014} project: {project_label} ")),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        );
    frame.render_widget(tabs, area);
}

fn draw_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .visible()
        .iter()
        .map(|entry| {
            let label = entry
                .math_kind
                .map(|kind| kind.to_string())
                .unwrap_or_else(|| entry.kind.to_string());
            let summary = entry.body.replace(['\n', '\r'], " ");
            ListItem::new(format!("{:<11} {:<6} {}", label, entry.state, summary))
        })
        .collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" entries "))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_right(frame: &mut Frame, app: &App, area: Rect) {
    match &app.right {
        RightPane::Detail => draw_detail(frame, app, area),
        RightPane::Trace(trace) => draw_trace(frame, trace, area),
    }
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let text = match app.selected_entry() {
        None => "no entry selected".to_string(),
        Some(entry) => {
            let mut lines = vec![
                format!("id:       {}", entry.id),
                format!("kind:     {}", entry.kind),
            ];
            if let Some(math_kind) = entry.math_kind {
                lines.push(format!("math:     {math_kind}"));
            }
            lines.push(format!("state:    {}", entry.state));
            lines.push(format!(
                "project:  {}",
                entry.project.as_deref().unwrap_or("global")
            ));
            lines.push(format!("revision: {}", entry.revision));
            if let Some(verification) = &entry.verification {
                lines.push(format!("verify:   {verification}"));
            }
            lines.push(String::new());
            lines.push(entry.body.clone());
            lines.join("\n")
        }
    };
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" detail (enter) "),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_trace(frame: &mut Frame, trace: &Trace, area: Rect) {
    let mut lines = Vec::new();
    for entry in &trace.entries {
        let marker = if entry.id == trace.root { "*" } else { " " };
        let summary = entry.body.replace(['\n', '\r'], " ");
        lines.push(Line::from(format!("{marker} {:<11} {summary}", entry.kind)));
    }
    if !trace.relations.is_empty() {
        lines.push(Line::from(""));
        for relation in &trace.relations {
            lines.push(Line::from(Span::raw(format!(
                "  {} --{}--> {}",
                &relation.from_id[..8.min(relation.from_id.len())],
                relation.kind,
                &relation.to_id[..8.min(relation.to_id.len())]
            ))));
        }
    }
    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" trace (t) "))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let open_questions = app
        .entries
        .iter()
        .filter(|entry| entry.kind == EntryKind::Question && entry.state == "open")
        .count();
    let ready_actions = app
        .entries
        .iter()
        .filter(|entry| entry.kind == EntryKind::Action && entry.state == "open")
        .count();
    let text = format!(
        "{}  |  entries: {}  open questions: {}  ready actions: {}",
        app.status,
        app.entries.len(),
        open_questions,
        ready_actions
    );
    let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}
