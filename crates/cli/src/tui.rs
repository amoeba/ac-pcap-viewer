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
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Tabs},
    Frame, Terminal,
};
use std::io::{self, Stdout};

use ac_parser::{messages::ParsedMessage, Direction as PktDirection, ParsedPacket};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Messages,
    Fragments,
}

pub struct App {
    pub messages: Vec<ParsedMessage>,
    pub packets: Vec<ParsedPacket>,
    pub current_tab: Tab,
    pub message_state: TableState,
    pub packet_state: TableState,
    pub message_sort: MessageSort,
    pub packet_sort: PacketSort,
    pub sort_ascending: bool,
    pub show_detail: bool,
    pub search_query: String,
    pub searching: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MessageSort {
    Id,
    Type,
    Direction,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PacketSort {
    Id,
    Sequence,
    Direction,
}

impl App {
    pub fn new(messages: Vec<ParsedMessage>, packets: Vec<ParsedPacket>) -> Self {
        let mut message_state = TableState::default();
        let mut packet_state = TableState::default();

        if !messages.is_empty() {
            message_state.select(Some(0));
        }
        if !packets.is_empty() {
            packet_state.select(Some(0));
        }

        Self {
            messages,
            packets,
            current_tab: Tab::Messages,
            message_state,
            packet_state,
            message_sort: MessageSort::Id,
            packet_sort: PacketSort::Id,
            sort_ascending: true,
            show_detail: false,
            search_query: String::new(),
            searching: false,
        }
    }

    pub fn filtered_messages(&self) -> Vec<&ParsedMessage> {
        let mut msgs: Vec<&ParsedMessage> = if self.search_query.is_empty() {
            self.messages.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.messages
                .iter()
                .filter(|m| m.message_type.to_lowercase().contains(&query))
                .collect()
        };

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

    pub fn filtered_packets(&self) -> Vec<&ParsedPacket> {
        let mut pkts: Vec<&ParsedPacket> = self.packets.iter().collect();

        pkts.sort_by(|a, b| {
            let cmp = match self.packet_sort {
                PacketSort::Id => a.id.cmp(&b.id),
                PacketSort::Sequence => a.header.sequence.cmp(&b.header.sequence),
                PacketSort::Direction => {
                    format!("{:?}", a.direction).cmp(&format!("{:?}", b.direction))
                }
            };
            if self.sort_ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });

        pkts
    }

    pub fn next(&mut self) {
        match self.current_tab {
            Tab::Messages => {
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
            Tab::Fragments => {
                let pkts = self.filtered_packets();
                if pkts.is_empty() {
                    return;
                }
                let i = match self.packet_state.selected() {
                    Some(i) => (i + 1).min(pkts.len() - 1),
                    None => 0,
                };
                self.packet_state.select(Some(i));
            }
        }
    }

    pub fn previous(&mut self) {
        match self.current_tab {
            Tab::Messages => {
                let i = match self.message_state.selected() {
                    Some(i) => i.saturating_sub(1),
                    None => 0,
                };
                self.message_state.select(Some(i));
            }
            Tab::Fragments => {
                let i = match self.packet_state.selected() {
                    Some(i) => i.saturating_sub(1),
                    None => 0,
                };
                self.packet_state.select(Some(i));
            }
        }
    }

    pub fn page_down(&mut self) {
        match self.current_tab {
            Tab::Messages => {
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
            Tab::Fragments => {
                let pkts = self.filtered_packets();
                if pkts.is_empty() {
                    return;
                }
                let i = match self.packet_state.selected() {
                    Some(i) => (i + 20).min(pkts.len() - 1),
                    None => 0,
                };
                self.packet_state.select(Some(i));
            }
        }
    }

    pub fn page_up(&mut self) {
        match self.current_tab {
            Tab::Messages => {
                let i = match self.message_state.selected() {
                    Some(i) => i.saturating_sub(20),
                    None => 0,
                };
                self.message_state.select(Some(i));
            }
            Tab::Fragments => {
                let i = match self.packet_state.selected() {
                    Some(i) => i.saturating_sub(20),
                    None => 0,
                };
                self.packet_state.select(Some(i));
            }
        }
    }

    pub fn cycle_sort(&mut self) {
        match self.current_tab {
            Tab::Messages => {
                self.message_sort = match self.message_sort {
                    MessageSort::Id => MessageSort::Type,
                    MessageSort::Type => MessageSort::Direction,
                    MessageSort::Direction => MessageSort::Id,
                };
            }
            Tab::Fragments => {
                self.packet_sort = match self.packet_sort {
                    PacketSort::Id => PacketSort::Sequence,
                    PacketSort::Sequence => PacketSort::Direction,
                    PacketSort::Direction => PacketSort::Id,
                };
            }
        }
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
                    KeyCode::Enter | KeyCode::Esc => {
                        app.searching = false;
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
                KeyCode::Tab => {
                    app.current_tab = match app.current_tab {
                        Tab::Messages => Tab::Fragments,
                        Tab::Fragments => Tab::Messages,
                    };
                }
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::PageDown | KeyCode::Char('d') => app.page_down(),
                KeyCode::PageUp | KeyCode::Char('u') => app.page_up(),
                KeyCode::Char('s') => app.cycle_sort(),
                KeyCode::Char('r') => app.toggle_sort_order(),
                KeyCode::Enter => app.show_detail = !app.show_detail,
                KeyCode::Char('/') => {
                    app.searching = true;
                    app.search_query.clear();
                }
                KeyCode::Esc => {
                    app.search_query.clear();
                    app.show_detail = false;
                }
                KeyCode::Home => match app.current_tab {
                    Tab::Messages => app.message_state.select(Some(0)),
                    Tab::Fragments => app.packet_state.select(Some(0)),
                },
                KeyCode::End => match app.current_tab {
                    Tab::Messages => {
                        let len = app.filtered_messages().len();
                        if len > 0 {
                            app.message_state.select(Some(len - 1));
                        }
                    }
                    Tab::Fragments => {
                        let len = app.filtered_packets().len();
                        if len > 0 {
                            app.packet_state.select(Some(len - 1));
                        }
                    }
                },
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

    let titles = vec!["Messages", "Fragments"];
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("AC PCAP Parser"),
        )
        .select(match app.current_tab {
            Tab::Messages => 0,
            Tab::Fragments => 1,
        })
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    if app.show_detail {
        render_detail(f, app, chunks[1]);
    } else {
        match app.current_tab {
            Tab::Messages => render_messages_table(f, app, chunks[1]),
            Tab::Fragments => render_packets_table(f, app, chunks[1]),
        }
    }

    let help = if app.searching {
        format!("Search: {}█  (Enter/Esc to finish)", app.search_query)
    } else {
        let sort_indicator = if app.sort_ascending { "↑" } else { "↓" };
        let sort_field = match app.current_tab {
            Tab::Messages => match app.message_sort {
                MessageSort::Id => "ID",
                MessageSort::Type => "Type",
                MessageSort::Direction => "Dir",
            },
            Tab::Fragments => match app.packet_sort {
                PacketSort::Id => "ID",
                PacketSort::Sequence => "Seq",
                PacketSort::Direction => "Dir",
            },
        };
        format!(
            "q:Quit Tab:Switch ↑↓/jk:Nav PgUp/PgDn:Page s:Sort({}{}) r:Reverse Enter:Detail /:Search",
            sort_field, sort_indicator
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

fn render_packets_table(f: &mut Frame, app: &mut App, area: Rect) {
    let pkts = app.filtered_packets();

    let header = Row::new(vec![
        Cell::from("ID").style(Style::default().fg(Color::Yellow)),
        Cell::from("Seq").style(Style::default().fg(Color::Yellow)),
        Cell::from("Dir").style(Style::default().fg(Color::Yellow)),
        Cell::from("Flags").style(Style::default().fg(Color::Yellow)),
        Cell::from("Size").style(Style::default().fg(Color::Yellow)),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD))
    .height(1);

    let rows: Vec<Row> = pkts
        .iter()
        .map(|p| {
            let dir_color = match p.direction {
                PktDirection::Send => Color::Cyan,
                PktDirection::Recv => Color::Green,
            };
            Row::new(vec![
                Cell::from(p.id.to_string()),
                Cell::from(p.header.sequence.to_string()),
                Cell::from(format!("{:?}", p.direction)).style(Style::default().fg(dir_color)),
                Cell::from(format!("{:08X}", p.header.flags.bits())),
                Cell::from(p.header.size.to_string()),
            ])
        })
        .collect();

    let title = format!("Fragments/Packets ({} total)", pkts.len());
    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(6),
            Constraint::Length(12),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    f.render_stateful_widget(table, area, &mut app.packet_state);
}

fn render_detail(f: &mut Frame, app: &App, area: Rect) {
    let content = match app.current_tab {
        Tab::Messages => {
            let msgs = app.filtered_messages();
            if let Some(idx) = app.message_state.selected() {
                if idx < msgs.len() {
                    let msg = msgs[idx];
                    serde_json::to_string_pretty(&msg.data).unwrap_or_else(|_| "Error".to_string())
                } else {
                    "No message selected".to_string()
                }
            } else {
                "No message selected".to_string()
            }
        }
        Tab::Fragments => {
            let pkts = app.filtered_packets();
            if let Some(idx) = app.packet_state.selected() {
                if idx < pkts.len() {
                    let pkt = pkts[idx];
                    serde_json::to_string_pretty(&pkt).unwrap_or_else(|_| "Error".to_string())
                } else {
                    "No packet selected".to_string()
                }
            } else {
                "No packet selected".to_string()
            }
        }
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Detail (Press Enter or Esc to close)"),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}
