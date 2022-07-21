///! For now this is for querying a driver. Likely in the future it will also be used to select older drivers, components and such.
use crate::nvapi::{xml::get_gpu_list, xml::XmlGpuEntry};
use crossterm::{
    self,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

pub async fn gpu_selector() -> Result<Option<XmlGpuEntry>, Box<dyn Error>> {
    // setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(125);
    let app = App::new();
    let res = run_app(&mut terminal, app.await, tick_rate);

    // restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

#[derive(Clone)]
struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

#[derive(Clone)]
struct App {
    all_items: Vec<XmlGpuEntry>,
    filtered_items: StatefulList<XmlGpuEntry>,
    input_mode: InputMode,
    query: String,
}

#[derive(Clone)]
enum InputMode {
    Normal,
    Search,
}

impl<'a> App {
    async fn new() -> App {
        let mut items: Vec<XmlGpuEntry> = get_gpu_list().await.unwrap();
        items.sort_by(|b, a| a.id.cmp(&b.id));
        let filtered_items = StatefulList::with_items(items.clone());
        App {
            all_items: items,
            filtered_items,
            input_mode: InputMode::Normal,
            query: String::new(),
        }
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<Option<XmlGpuEntry>, Box<dyn Error>> {
    let last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(None),
                        KeyCode::Char('s') => {
                            app.input_mode = InputMode::Search;
                            app.query.clear();
                            app.filtered_items.items = Vec::new();
                        }
                        KeyCode::Left => app.filtered_items.unselect(),
                        KeyCode::Down => app.filtered_items.next(),
                        KeyCode::Up => app.filtered_items.previous(),
                        KeyCode::Enter => {
                            if let Some(item) = app.filtered_items.state.selected() {
                                return Ok(Some(
                                    app.filtered_items.items.get(item).unwrap().clone(),
                                ));
                            }
                        }
                        _ => {}
                    },
                    InputMode::Search => match key.code {
                        KeyCode::Enter => {
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char(c) => {
                            app.query.push(c);
                            app.filtered_items.items = Vec::new();
                            for gpu in app.all_items.iter() {
                                if gpu.name.contains(&app.query) {
                                    app.filtered_items.items.push(gpu.clone());
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            app.query.pop();
                            app.filtered_items.items = Vec::new();
                            for gpu in app.all_items.iter() {
                                if gpu.name.contains(&app.query) {
                                    app.filtered_items.items.push(gpu.clone());
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(85),
                Constraint::Percentage(10),
                Constraint::Percentage(5),
            ]
            .as_ref(),
        )
        .split(f.size());

    let mut items: Vec<ListItem> = app
        .filtered_items
        .items
        .iter()
        .map(|i| {
            ListItem::new(i.name.clone()).style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    // Fix some items showing up twice, not all though?
    // TODO: Figure out why this happens
    items.dedup();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select Your GPU"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("s", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to search, Arrow keys to navigate and Enter to select."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Search => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop searching, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to confirm your search"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[2]);

    let query = Paragraph::new(Spans::from(app.query.clone()))
        .block(Block::default().borders(Borders::ALL).title("Search"))
        .style(Style::default().fg(Color::Black).bg(Color::White));

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.filtered_items.state);
    f.render_widget(query, chunks[1]);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Search => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.query.len() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
    }
}
