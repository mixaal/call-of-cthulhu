use call_of_cthulhu::game;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    game::play()
}
