use std::io;

use crossterm::event::KeyCode;
use ratatui::{
    Terminal,
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List},
};

use crate::{
    engine::{config, fs},
    screens::play::GameEvent,
};

pub struct AchievementScreen {
    pub(crate) width: usize,
    pub(crate) height: usize,
    image: Vec<Vec<(u8, u8, u8)>>,
}

const ACHIEVEMENTS: [&'static str; 5] = [
    "Achievement 1: First Steps",
    "Achievement 2: Explorer",
    "Achievement 3: Master of Screens",
    "Achievement 4: Completionist",
    "Achievement 5: Unlocked All Secrets",
];

impl AchievementScreen {
    pub fn new(
        width: usize,
        height: usize,
        config: &config::Config,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let image = fs::load_achievements_screen_image(width as u16, height as u16, config)?;
        Ok(Self {
            width,
            height,
            image,
        })
    }

    pub fn render(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        terminal.draw(|f| {
            let area = f.area();

            // Draw intro image across full terminal
            for (y, row) in self.image.iter().enumerate() {
                for (x, (r, g, b)) in row.iter().enumerate() {
                    if y < area.height as usize && x < area.width as usize {
                        if let Some(cell) = f.buffer_mut().cell_mut((x as u16, y as u16)) {
                            cell.set_bg(Color::Rgb(*r, *g, *b));
                        }
                    }
                }
            }

            let list = List::new(ACHIEVEMENTS.iter().map(|a| Line::from(*a)))
                .block(Block::default().borders(Borders::ALL).title("Achievements"))
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );

            f.render_widget(list, area);
        })?;

        Ok(())
    }

    pub fn key_event(&mut self, key_code: KeyCode) -> Option<GameEvent> {
        match key_code {
            KeyCode::Enter | KeyCode::Esc => {
                Some(GameEvent::Exit) // just exit the graph view on Enter
            }
            _ => None,
        }
    }
}
