use call_of_cthulhu::{
    config,
    game::{self, GameEvent, GameState},
    gfx::{self, ScreenRenderer},
    graph, validate,
};
use crossterm::event::{self, Event};

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = config::Config::load()?;
    // validate data files
    validate::validate_screens(&cfg)?;
    // load screen graph
    let mut game_graph = graph::GameGraph::load(&cfg);
    // Initialize terminal
    let mut terminal = gfx::init()?;

    let dim = terminal.size()?;
    //println!("Terminal size: {}x{}", dim.width, dim.height);

    let current_screen = cfg.get_screen();
    game_graph.visit(current_screen);
    let mut screen: Box<dyn ScreenRenderer<GameEvent>> = Box::new(game::GameScreen::new(
        current_screen,
        dim.width,
        dim.height,
        &cfg,
    )?);

    let mut state = GameState::Playing;

    loop {
        if event::poll(std::time::Duration::from_millis(5))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.code == crossterm::event::KeyCode::Esc {
                    break;
                }

                match state {
                    GameState::Playing => {
                        if let Some(event) = screen.key_event(key_event.code) {
                            match event {
                                GameEvent::NewScreen(screen_no) => {
                                    game_graph.visit(screen_no);
                                    screen = Box::new(game::GameScreen::new(
                                        screen_no, dim.width, dim.height, &cfg,
                                    )?);
                                }
                                GameEvent::Exit => break,
                                GameEvent::Ending => state = GameState::Ending,
                            }
                        }
                    }
                    GameState::Ending => {
                        // do nothing, just show the graph
                    }
                    GameState::Intro => {
                        // do nothing, just show the intro screen
                    }
                }
            }
        }
        if state == GameState::Ending {
            game_graph.render(&mut terminal)?;
        } else {
            gfx::render(&mut terminal, &mut screen)?;
        }
        //println!("State: {:?}", state);
    }

    // Restore terminal
    gfx::shutdown(terminal)?;
    Ok(())
}
