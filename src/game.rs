use std::{collections::HashMap, hash::Hash, time::Instant};

use serde::Deserialize;

use crate::{
    config, fs,
    gfx::{self, ScreenRenderer},
};

#[derive(Deserialize)]
pub(crate) struct GameActions {
    // next screens with action text and next screen index
    pub(crate) next: HashMap<String, usize>,
    pub(crate) location: Option<String>,
    pub(crate) ending: Option<bool>,
}

impl GameActions {
    pub(crate) fn ending() -> Vec<(String, usize)> {
        vec![("KONEC".to_string(), 0)]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameEvent {
    Exit,
    NewScreen(usize),
    Ending,
}
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Intro,
    Playing,
    Ending,
}

struct InnerConfig {
    pub(crate) scale_quality: bool,
    pub(crate) notifications: bool,
}

pub struct GameScreen {
    screen_no: usize,
    term_width: u16,
    term_height: u16,
    text_helper: gfx::TextHelper,
    timer: Instant,
    total_time_to_write: f32,
    actions: Vec<(String, usize)>,
    inner_config: InnerConfig,
    menu_selection: usize,
    image_names: Vec<String>,
    ending_screen: bool,
    location: Option<String>,
}

impl GameScreen {
    pub fn new(
        screen_no: usize,
        term_width: u16,
        term_height: u16,
        config: &config::Config,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let text_helper = gfx::TextHelper::with_text(
            config.text_speed,
            fs::read_text(screen_no, config)
                .unwrap_or(format!("Error reading screen {}", screen_no)),
        );
        let total_time_to_write = text_helper.time_to_finish();
        let action_desc = fs::read_actions(screen_no, config)?;
        let ending_screen = action_desc.ending.unwrap_or(false);
        let location = action_desc.location.clone();
        let actions = if ending_screen {
            GameActions::ending()
        } else {
            action_desc
                .next
                .into_iter()
                .collect::<Vec<(String, usize)>>()
        };

        let image_names = fs::get_image_names_for_screen(screen_no, config)?;
        if image_names.is_empty() {
            return Err(format!("No images found for screen {}", screen_no).into());
        }

        let send_notifications = config.notifications.unwrap_or(false);
        if send_notifications {
            Self::inform_location_change(location.clone());
            if ending_screen {
                Self::send_notification("Ending screen  🎉");
            }
        }
        Ok(Self {
            screen_no,
            term_width,
            term_height,
            text_helper,
            total_time_to_write,
            timer: Instant::now(),
            actions,
            inner_config: InnerConfig {
                scale_quality: config.scale_quality,
                notifications: send_notifications,
            },
            menu_selection: 0,
            image_names,
            ending_screen,
            location,
        })
    }

    fn inform_location_change(location: Option<String>) {
        if let Some(location) = location {
            Self::send_notification(&format!("Location: {}", location));
        }
    }

    fn send_notification(text: &str) {
        notify_rust::Notification::new()
            .summary("Call of Cthulhu")
            .body(text)
            .show()
            .ok();
    }
}

fn actions_text(actions: &Vec<(String, usize)>, idx: usize) -> String {
    let mut contents = String::new();
    let l = actions.len();
    if l == 0 {
        // no actions, most likely ending screen
        return contents;
    }
    let idx = idx % l;
    let mut i = 0;
    for (action_text, next_screen) in actions.iter() {
        if i == idx {
            contents.push_str(&format!("---> {}\n", action_text));
        } else {
            contents.push_str(&format!("     {}\n", action_text));
        }
        i += 1;
    }
    contents
}

impl ScreenRenderer<GameEvent> for GameScreen {
    fn render(&mut self) -> Vec<Vec<(u8, u8, u8)>> {
        // if multiple images are present for the given screen...
        let l = self.image_names.len();
        // ... compute the time that should be spent on each image ...
        let per_image_time = self.total_time_to_write / l as f32;
        // ... and cycle the images on screen
        let idx = (self.timer.elapsed().as_secs_f64() / per_image_time as f64) as usize % l;
        let tw = self.term_width - self.text_window_sz();
        if let Ok((_, _, screen)) = fs::read_image(
            &self.image_names[idx],
            tw,
            self.term_height,
            self.inner_config.scale_quality,
        ) {
            return screen;
        } else {
            // blue screen of death
            return vec![vec![(0, 0, 255); tw as usize]; self.term_height as usize];
        }
    }

    fn text(&mut self) -> String {
        let text = self.text_helper.get_text().unwrap_or_default();
        if let Some(when) = self.text_helper.text_reached_end() {
            let actions_text = actions_text(&self.actions, self.menu_selection);
            format!("{}\n\n{}", text, actions_text)
        } else {
            text
        }
    }

    fn text_window_sz(&self) -> u16 {
        self.term_width / 4
    }

    fn key_event(&mut self, key_code: crossterm::event::KeyCode) -> Option<GameEvent> {
        match key_code {
            crossterm::event::KeyCode::Esc => Some(GameEvent::Exit),
            crossterm::event::KeyCode::Down => {
                if self.menu_selection + 1 < self.actions.len() {
                    self.menu_selection += 1;
                }
                None
            }
            crossterm::event::KeyCode::Up => {
                if self.menu_selection > 0 {
                    self.menu_selection -= 1;
                }
                None
            }
            crossterm::event::KeyCode::Enter => {
                if let Some((_, next_screen)) = self.actions.get(self.menu_selection) {
                    if self.ending_screen {
                        return Some(GameEvent::Ending);
                    } else {
                        return Some(GameEvent::NewScreen(*next_screen));
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
