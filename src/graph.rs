use std::{collections::HashMap, hash::Hash, io};

use ratatui::{Terminal, prelude::CrosstermBackend};

use crate::{
    config, fs,
    game::{self, GameEvent},
    gfx,
};

pub struct GameGraph {
    pub(crate) graph: HashMap<usize, Vec<usize>>,
    pub(crate) visited: HashMap<usize, usize>, // how many times each screen was visited
}

impl GameGraph {
    fn new() -> Self {
        Self {
            graph: HashMap::new(),
            visited: HashMap::new(),
        }
    }

    fn add_screen(&mut self, screen_no: usize, actions: &[(String, usize)]) {
        let next_screens = actions
            .iter()
            .map(|(_, next_screen)| *next_screen)
            .collect();
        self.graph.insert(screen_no, next_screens);
    }

    pub fn visit(&mut self, screen_no: usize) {
        self.visited
            .entry(screen_no)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    pub fn load(config: &config::Config) -> Self {
        let mut g = Self::new();
        for screen_no in 0..=111 {
            if let Ok(actions) = fs::read_actions(screen_no, config) {
                g.add_screen(screen_no, &actions.next.into_iter().collect::<Vec<_>>());
            }
        }
        g
    }

    pub fn render(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        terminal.draw(|f| {
            let area = f.area();
            let block = ratatui::widgets::Block::default()
                .title("Game Graph")
                .borders(ratatui::widgets::Borders::ALL);
            f.render_widget(block, area);

            let mut y = 1;
            let mut x = 1;
            let mut col_width = area.width / 2 - 2;

            for (screen_no, next_screens) in &self.graph {
                if y >= area.height - 1 && x == 1 {
                    x = (area.width / 2) + 1;
                    y = 1;
                }

                let visit_count = self.visited.get(screen_no).copied().unwrap_or(0);
                let style = if visit_count == 0 {
                    ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray)
                } else {
                    let color = match visit_count {
                        1..=2 => ratatui::style::Color::Blue,
                        3..=5 => ratatui::style::Color::Cyan,
                        6..=10 => ratatui::style::Color::Green,
                        11..=20 => ratatui::style::Color::Yellow,
                        21..=50 => ratatui::style::Color::Magenta,
                        _ => ratatui::style::Color::Red,
                    };
                    ratatui::style::Style::default().fg(color)
                };

                let text = if visit_count == 0 {
                    "".to_string()
                } else {
                    format!(
                        "Screen {}: {:?} ({}x)",
                        screen_no, next_screens, visit_count
                    )
                };
                let line = ratatui::text::Line::raw(text).style(style);
                f.render_widget(
                    ratatui::widgets::Paragraph::new(line),
                    ratatui::layout::Rect {
                        x,
                        y,
                        width: col_width,
                        height: 1,
                    },
                );
                y += 1;
            }
        })?;
        Ok(())
    }
}
