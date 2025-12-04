use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use std::io::{self, Stdout};

use lib::{messages::ParsedMessage, ParsedPacket};

use crate::filter;

pub struct App {
    pub messages: Vec<ParsedMessage>,
    pub message_state: TableState,
    pub message_sort: MessageSort,
    pub sort_ascending: bool,
    pub show_detail: bool,
    pub search_query: String,
    pub searching: bool,
    pub search_mode: SearchMode,
    pub filter_opcode: Option<u32>,
    pub filter_direction: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Type,
    Opcode,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MessageSort {
    Id,
    Type,
    Direction,
}

impl App {
    pub fn new(messages: Vec<ParsedMessage>, _packets: Vec<ParsedPacket>) -> Self {
        let mut message_state = TableState::default();

        if !messages.is_empty() {
            message_state.select(Some(0));
        }

        Self {
            messages,
            message_state,
            message_sort: MessageSort::Id,
            sort_ascending: true,
            show_detail: false,
            search_query: String::new(),
            searching: false,
            search_mode: SearchMode::Type,
            filter_opcode: None,
            filter_direction: None,
        }
    }

    pub fn filtered_messages(&self) -> Vec<&ParsedMessage> {
        let mut msgs: Vec<&ParsedMessage> = self
            .messages
            .iter()
            .filter(|m| {
                // Type filter
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    if !m.message_type.to_lowercase().contains(&query) {
                        return false;
                    }
                }

                // Opcode filter
                if let Some(oc) = self.filter_opcode {
                    if let Some(msg_opcode) = filter::opcode_str_to_u32(&m.opcode) {
                        if msg_opcode != oc {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                // Direction filter
                if let Some(ref dir) = self.filter_direction {
                    if m.direction != *dir {
                        return false;
                    }
                }

                true
            })
            .collect();

        msgs.sort_by(|a, b| {
            let cmp = match self.message_sort {
                MessageSort::Id => a.id.cmp(&b.id),
                MessageSort::Type => a.message_type.cmp(&b.message_type),
                MessageSort::Direction => a.direction.cmp(&b.direction),
            };
            if self.sort_ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });

        msgs
    }

    pub fn next(&mut self) {
        let msgs = self.filtered_messages();
        if msgs.is_empty() {
            return;
        }
        let i = match self.message_state.selected() {
            Some(i) => (i + 1).min(msgs.len() - 1),
            None => 0,
        };
        self.message_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.message_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.message_state.select(Some(i));
    }

    pub fn page_down(&mut self) {
        let msgs = self.filtered_messages();
        if msgs.is_empty() {
            return;
        }
        let i = match self.message_state.selected() {
            Some(i) => (i + 20).min(msgs.len() - 1),
            None => 0,
        };
        self.message_state.select(Some(i));
    }

    pub fn page_up(&mut self) {
        let i = match self.message_state.selected() {
            Some(i) => i.saturating_sub(20),
            None => 0,
        };
        self.message_state.select(Some(i));
    }

    pub fn cycle_sort(&mut self) {
        self.message_sort = match self.message_sort {
            MessageSort::Id => MessageSort::Type,
            MessageSort::Type => MessageSort::Direction,
            MessageSort::Direction => MessageSort::Id,
        };
    }

    pub fn toggle_sort_order(&mut self) {
        self.sort_ascending = !self.sort_ascending;
    }
}

pub fn run_tui(messages: Vec<ParsedMessage>, packets: Vec<ParsedPacket>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(messages, packets);
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if app.searching {
                match key.code {
                    KeyCode::Enter => {
                        app.searching = false;
                        match app.search_mode {
                            SearchMode::Type => {
                                // Type search stays in search_query
                            }
                            SearchMode::Opcode => {
                                // Parse opcode and apply filter
                                if let Ok(oc) = filter::parse_opcode_filter(&app.search_query) {
                                    app.filter_opcode = Some(oc);
                                }
                                app.search_query.clear();
                            }
                        }
                        // Reset to first row when filter changes
                        app.message_state.select(Some(0));
                    }
                    KeyCode::Esc => {
                        app.searching = false;
                        app.search_query.clear();
                    }
                    KeyCode::Backspace => {
                        app.search_query.pop();
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                    }
                    _ => {}
                }
                continue;
            }

            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::PageDown | KeyCode::Char('d') => app.page_down(),
                KeyCode::PageUp | KeyCode::Char('u') => app.page_up(),
                KeyCode::Char('s') => app.cycle_sort(),
                KeyCode::Char('r') => app.toggle_sort_order(),
                KeyCode::Enter => {
                    // Ensure something is selected before showing detail
                    if app.message_state.selected().is_none() && !app.filtered_messages().is_empty()
                    {
                        app.message_state.select(Some(0));
                    }
                    app.show_detail = !app.show_detail;
                }
                KeyCode::Char('/') => {
                    app.searching = true;
                    app.search_mode = SearchMode::Type;
                    app.search_query.clear();
                }
                KeyCode::Char('f') => {
                    // Toggle direction filter (cycle through Send, Recv, None)
                    app.filter_direction = match app.filter_direction.as_deref() {
                        None => Some("Send".to_string()),
                        Some("Send") => Some("Recv".to_string()),
                        Some("Recv") => None,
                        _ => None,
                    };
                    // Reset to first row when filter changes
                    app.message_state.select(Some(0));
                }
                KeyCode::Char('o') => {
                    // Enter opcode search mode
                    app.searching = true;
                    app.search_mode = SearchMode::Opcode;
                    app.search_query.clear();
                }
                KeyCode::Esc => {
                    app.search_query.clear();
                    app.show_detail = false;
                }
                KeyCode::Home => app.message_state.select(Some(0)),
                KeyCode::End => {
                    let len = app.filtered_messages().len();
                    if len > 0 {
                        app.message_state.select(Some(len - 1));
                    }
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title("AC PCAP Parser - Messages");
    f.render_widget(title_block, chunks[0]);

    if app.show_detail {
        render_detail(f, app, chunks[1]);
    } else {
        render_messages_table(f, app, chunks[1]);
    }

    let help = if app.searching {
        let mode_label = match app.search_mode {
            SearchMode::Type => "Type Search",
            SearchMode::Opcode => "OpCode Filter (0xF7B1 or 63409)",
        };
        format!(
            "{}: {}█  (Enter/Esc to finish)",
            mode_label, app.search_query
        )
    } else {
        let sort_indicator = if app.sort_ascending { "↑" } else { "↓" };
        let sort_field = match app.message_sort {
            MessageSort::Id => "ID",
            MessageSort::Type => "Type",
            MessageSort::Direction => "Dir",
        };

        let dir_filter = match app.filter_direction.as_deref() {
            None => "All".to_string(),
            Some("Send") => "Send".to_string(),
            Some("Recv") => "Recv".to_string(),
            _ => "All".to_string(),
        };

        let opcode_filter = match app.filter_opcode {
            None => "None".to_string(),
            Some(oc) => format!("{oc:04X}"),
        };

        format!(
            "q:Quit ↑↓/jk:Nav PgUp/PgDn:Page s:Sort({sort_field}{sort_indicator}) r:Reverse Enter:Detail /:Search f:Dir({dir_filter}) o:OpCode({opcode_filter})"
        )
    };

    let status = Paragraph::new(help).block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(status, chunks[2]);
}

fn render_messages_table(f: &mut Frame, app: &mut App, area: Rect) {
    let msgs = app.filtered_messages();

    let header = Row::new(vec![
        Cell::from("ID").style(Style::default().fg(Color::Yellow)),
        Cell::from("Type").style(Style::default().fg(Color::Yellow)),
        Cell::from("Dir").style(Style::default().fg(Color::Yellow)),
        Cell::from("OpCode").style(Style::default().fg(Color::Yellow)),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD))
    .height(1);

    let rows: Vec<Row> = msgs
        .iter()
        .map(|m| {
            let dir_color = match m.direction.as_str() {
                "Send" => Color::Cyan,
                "Recv" => Color::Green,
                _ => Color::White,
            };
            Row::new(vec![
                Cell::from(m.id.to_string()),
                Cell::from(m.message_type.clone()),
                Cell::from(m.direction.clone()).style(Style::default().fg(dir_color)),
                Cell::from(m.opcode.clone()),
            ])
        })
        .collect();

    let title = format!(
        "Messages ({} total, {} shown)",
        app.messages.len(),
        msgs.len()
    );
    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Min(30),
            Constraint::Length(6),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    f.render_stateful_widget(table, area, &mut app.message_state);
}

fn render_detail(f: &mut Frame, app: &App, area: Rect) {
    let msgs = app.filtered_messages();
    let content = if let Some(idx) = app.message_state.selected() {
        if idx < msgs.len() {
            let msg = msgs[idx];
            serde_json::to_string_pretty(&msg.data).unwrap_or_else(|_| "Error".to_string())
        } else {
            "No message selected".to_string()
        }
    } else {
        "No message selected".to_string()
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Detail (Press Enter or Esc to close)"),
        )
        .style(Style::default().fg(Color::Cyan));

    f.render_widget(paragraph, area);
}
