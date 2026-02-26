use std::io;

use crossterm::event::KeyCode;
use ratatui::{
    Terminal,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListState, Paragraph},
};

use crate::{config, fs};

pub struct IntroScreen {
    pub(crate) width: usize,
    pub(crate) height: usize,
    internal_item_selected: usize,
    menu_item_selected: Option<usize>,
    intro_image: Vec<Vec<(u8, u8, u8)>>,
    list_state: ListState,
}

pub const NEW_GAME: usize = 0;
pub const CONTINUE: usize = 1;
pub const SAVE: usize = 2;
pub const ACHIEVEMENTS: usize = 3;
pub const CREDITS: usize = 4;
pub const EXIT: usize = 5;

const MENU_ITEMS: [&'static str; 6] = [
    "New Game",
    "Continue Saved...",
    "Save",
    "Achievements",
    "Credits",
    "Exit",
];
impl IntroScreen {
    pub fn new(
        width: usize,
        height: usize,
        config: &config::Config,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let intro_image = fs::load_intro_screen_image(width as u16, height as u16, config)?;
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Ok(Self {
            width,
            height,
            internal_item_selected: 0,
            menu_item_selected: None,
            intro_image,
            list_state,
        })
    }

    pub fn render(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        terminal.draw(|f| {
            let area = f.area();

            // Draw intro image across full terminal
            for (y, row) in self.intro_image.iter().enumerate() {
                for (x, (r, g, b)) in row.iter().enumerate() {
                    if y < area.height as usize && x < area.width as usize {
                        if let Some(cell) = f.buffer_mut().cell_mut((x as u16, y as u16)) {
                            cell.set_bg(Color::Rgb(*r, *g, *b));
                        }
                    }
                }
            }

            // Menu overlay in bottom half
            let menu_area = ratatui::layout::Rect {
                x: area.x,
                y: area.y + 4 * area.height / 5,
                width: area.width,
                height: area.height / 5,
            };

            let menu = List::new(MENU_ITEMS)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");
            f.render_stateful_widget(menu, menu_area, &mut self.list_state.clone());
        })?;

        Ok(())
    }

    pub fn get_selected_item(&self) -> Option<usize> {
        self.menu_item_selected
    }

    pub fn key_event(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Up => {
                self.menu_item_selected = None; // reset menu selection when navigating
                if self.internal_item_selected > 0 {
                    self.internal_item_selected -= 1;
                    self.list_state.select(Some(self.internal_item_selected));
                }
            }
            KeyCode::Down => {
                self.menu_item_selected = None; // reset menu selection when navigating
                if self.internal_item_selected < MENU_ITEMS.len() - 1 {
                    self.internal_item_selected += 1;
                    self.list_state.select(Some(self.internal_item_selected));
                }
            }
            KeyCode::Enter => {
                self.menu_item_selected = Some(self.internal_item_selected);
            }
            _ => self.menu_item_selected = None, // reset menu selection when navigating
        }
    }
}
