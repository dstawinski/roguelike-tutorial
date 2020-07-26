use crate::constants::*;
use crate::structs::*;

use tcod::map::Map as FovMap;

pub fn setup_fov(fov: &mut FovMap, game: &Game) {
    // populate the FOV map, according to the generated map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            fov.set(
                x,
                y,
                !game.map[x as usize][y as usize].block_sight,
                !game.map[x as usize][y as usize].blocked,
            );
        }
    }
}
