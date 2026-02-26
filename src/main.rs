use call_of_cthulhu::{
    config,
    game::{self, GameState},
    gfx::{self, ScreenRenderer},
    validate,
};
use crossterm::event::{self, Event};

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = config::Config::load()?;
    // validate data files
    validate::validate_screens(&cfg)?;
    // Initialize terminal
    let mut terminal = gfx::init()?;

    let dim = terminal.size()?;
    //println!("Terminal size: {}x{}", dim.width, dim.height);

    let current_screen = cfg.get_screen();

    let mut screen: Box<dyn ScreenRenderer<GameState>> = Box::new(game::GameScreen::new(
        current_screen,
        dim.width,
        dim.height,
        &cfg,
    )?);
    loop {
        if event::poll(std::time::Duration::from_millis(5))? {
            if let Event::Key(key_event) = event::read()? {
                if let Some(state) = screen.key_event(key_event.code) {
                    match state {
                        GameState::NewScreen(screen_no) => {
                            screen = Box::new(game::GameScreen::new(
                                screen_no, dim.width, dim.height, &cfg,
                            )?);
                        }
                        GameState::Exit => break,
                        GameState::Ending => {
                            screen =
                                Box::new(game::GameScreen::new(0, dim.width, dim.height, &cfg)?);
                        }
                    }
                }
            }
        }

        gfx::render(&mut terminal, &mut screen)?;
    }

    // Restore terminal
    gfx::shutdown(terminal)?;
    Ok(())
}
