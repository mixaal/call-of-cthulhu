use std::{error::Error, io, time::Instant};

use crossterm::{
    event::KeyCode,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

// initialize the gfx module
pub fn init() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

// shutdown the gfx module
pub fn shutdown(
    mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

pub trait ScreenRenderer<T> {
    // Renders the screen with the given time.
    //
    // # Returns
    //
    // A vector of colors representing the screen.
    fn render(&mut self) -> Vec<Vec<(u8, u8, u8)>>;

    fn text(&mut self) -> String;

    fn text_window_sz(&self) -> u16;

    fn key_event(&mut self, key_code: KeyCode) -> Option<T>;
}

// ongoing render loop
pub fn render<T>(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    renderer: &mut Box<dyn ScreenRenderer<T>>,
) -> io::Result<()> {
    let term_sz = terminal.size().expect("can't get terminal size");

    let colors = renderer.render();
    let wh = renderer.text_window_sz();

    let max_w = term_sz.width / 4; // Split the terminal width into two parts 1:3

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Horizontal) // Vertical split
            .constraints([Constraint::Length(max_w), Constraint::Min(0)].as_ref())
            .split(f.area());

        // Render the graphics window on the right
        let graphics_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                colors
                    .iter()
                    .map(|_| Constraint::Length(1))
                    .collect::<Vec<_>>(),
            )
            .split(chunks[1]);

        for (i, row) in colors.iter().enumerate() {
            if i >= graphics_chunks.len() {
                break; // Prevent rendering beyond the available space
            }
            let spans = row
                .iter()
                .map(|&(r, g, b)| {
                    let color = Color::Rgb(r, g, b);
                    Span::styled("█", Style::default().fg(color))
                })
                .collect::<Vec<_>>();
            let paragraph = Paragraph::new(Line::from(spans));
            f.render_widget(paragraph, graphics_chunks[i]);
        }

        // Render the text window on the left
        let bottom_text = renderer.text();

        let window_block = Block::default()
            .title("Text Window")
            .style(Style::default().fg(Color::White))
            .border_style(Style::default().fg(Color::Cyan))
            .borders(ratatui::widgets::Borders::ALL);

        let centered_text = Paragraph::new(bottom_text)
            .style(Style::default().fg(Color::White))
            .alignment(ratatui::layout::Alignment::Center)
            .block(window_block);

        f.render_widget(centered_text, chunks[0]);
    })?;
    // }
    Ok(())
}

// TextHelper manages text display
pub struct TextHelper {
    pub text: Option<String>,
    pub text_delay_timer: Instant,
    pub write_speed: f64,
    pub end_of_writing: Option<Instant>,
}

impl TextHelper {
    pub fn new(write_speed: f64) -> Self {
        let text_delay_timer = Instant::now();
        Self {
            text: None,
            write_speed,
            text_delay_timer,
            end_of_writing: None,
        }
    }

    pub fn with_text(write_speed: f64, text: String) -> Self {
        let text_delay_timer = Instant::now();
        Self {
            text: Some(text),
            write_speed,
            text_delay_timer,
            end_of_writing: None,
        }
    }

    pub fn new_text(&mut self, text: String) {
        self.text = Some(text);
        self.text_delay_timer = Instant::now();
        self.end_of_writing = None;
    }

    pub fn text_reached_end(&mut self) -> Option<Instant> {
        if self.text.is_none() {
            return None;
        }
        let text = self.text.as_ref().unwrap();
        let is_end = self.chars_to_show() >= text.len();
        if is_end && self.end_of_writing.is_none() {
            self.end_of_writing = Some(Instant::now());
        }
        self.end_of_writing
    }

    pub fn get_text(&mut self) -> Option<String> {
        if let Some(text) = &self.text {
            let chars_to_show = self.chars_to_show();
            if chars_to_show >= text.len() {
                return Some(text.clone());
            }
            // Show only the characters that have been "typed" so far
            let text: String = text.chars().take(chars_to_show).collect();
            Some(text)
        } else {
            None
        }
    }

    fn chars_to_show(&self) -> usize {
        let elapsed_time = self.text_delay_timer.elapsed().as_secs_f64();

        (elapsed_time * self.write_speed) as usize
    }

    pub(crate) fn time_to_finish(&self) -> f32 {
        if self.text.is_none() {
            return 0.0;
        }
        let text = self.text.as_ref().unwrap();
        let chars_left = text.len() - self.chars_to_show();
        let time_left = chars_left as f64 / self.write_speed;
        time_left as f32
    }
}

pub trait Updater {
    fn update(&mut self);
}

pub struct Blink {
    blink_interval: f64,
    last_blink_time: Instant,
    pub is_visible: bool,
}

impl Blink {
    pub fn new(blink_interval: f64) -> Self {
        Self {
            blink_interval,
            last_blink_time: Instant::now(),
            is_visible: true,
        }
    }
}

impl Updater for Blink {
    fn update(&mut self) {
        if self.last_blink_time.elapsed().as_secs_f64() > self.blink_interval {
            self.is_visible = !self.is_visible;
            self.last_blink_time = Instant::now();
        }
    }
}

pub struct FirstTime {
    pub first_time: bool,
}

impl FirstTime {
    pub fn new() -> Self {
        Self { first_time: true }
    }

    pub fn isit(&self) -> bool {
        self.first_time
    }
}

impl Updater for FirstTime {
    fn update(&mut self) {
        if self.first_time {
            self.first_time = false;
        }
    }
}

// timer, can be periodical or one-time
pub struct Timer {
    start_time: Option<Instant>,
    trigger_time: f64,
    periodical: bool,
}
impl Timer {
    pub fn new(trigger_time: f64, periodical: bool) -> Self {
        Self {
            trigger_time,
            start_time: None,
            periodical,
        }
    }

    pub fn reset(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn is_trigger(&mut self) -> bool {
        if let Some(start_time) = self.start_time {
            let elapsed_time = start_time.elapsed().as_secs_f64();
            if self.periodical {
                if elapsed_time > self.trigger_time {
                    self.start_time = Some(Instant::now());
                    return true;
                }
            } else {
                if elapsed_time > self.trigger_time {
                    return true;
                }
            }
        }
        return false;
    }
}
