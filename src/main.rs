mod constants;
mod fov;
mod map;
mod render_all;
mod structs;

use crate::constants::*;
use crate::fov::*;
use crate::map::*;
use crate::render_all::*;
use crate::structs::*;

use tcod::colors::*;
use tcod::console::*;
use tcod::map::Map as FovMap;

fn main() {
    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust/libtcod tutorial")
        .init();

    let mut tcod = Tcod {
        root,
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
    };

    tcod::system::set_fps(LIMIT_FPS);

    // Game objects
    // create object representing the player
    let mut player = Object::new(0, 0, '@', "player", WHITE, true);
    player.alive = true;
    player.fighter = Some(Fighter {
        max_hp: 30,
        hp: 30,
        defense: 2,
        power: 5,
    });
    let mut objects = vec![player];

    let mut game = Game {
        map: make_map(&mut objects),
    };

    setup_fov(&mut tcod.fov, &game);

    let mut previous_player_position = (-1, -1);
    // Game loop
    while !tcod.root.window_closed() {
        tcod.con.clear();
        let fov_recompute = previous_player_position != objects[PLAYER].pos();
        render_all(&mut tcod, &mut game, &objects, fov_recompute);
        tcod.root.flush();
        // handle keys and exit game if needed
        previous_player_position = objects[PLAYER].pos();
        let map = &game.map;
        let player_action = handle_keys(&mut tcod, &map, &mut objects);
        if player_action == PlayerAction::Exit {
            break;
        }
        if objects[PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
            for object in &objects {
                // only if object is not player
                if (object as *const _) != (&objects[PLAYER] as *const _) {
                    println!("The {} growls!", object.name);
                }
            }
        }
    }
}

fn handle_keys(tcod: &mut Tcod, map: &Map, mut objects: &mut [Object]) -> PlayerAction {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    use PlayerAction::*;
    let key = tcod.root.wait_for_keypress(true);
    let player_alive = objects[PLAYER].alive;
    match (key, key.text(), player_alive) {
        // Movement
        (Key { code: Up, .. }, _, true) => {
            player_move_or_attack(0, -1, map, &mut objects);
            TookTurn
        }
        (Key { code: Down, .. }, _, true) => {
            player_move_or_attack(0, 1, map, &mut objects);
            TookTurn
        }
        (Key { code: Left, .. }, _, true) => {
            player_move_or_attack(-1, 0, map, &mut objects);
            TookTurn
        }
        (Key { code: Right, .. }, _, true) => {
            player_move_or_attack(1, 0, map, &mut objects);
            TookTurn
        }
        // Fullscreen
        (
            Key {
                code: Enter,
                alt: true,
                ..
            },
            _,
            _,
        ) => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
            DidntTakeTurn
        }
        // Exit
        (Key { code: Escape, .. }, _, _) => Exit, // exit game

        _ => DidntTakeTurn,
    }
}

fn move_by(id: usize, dx: i32, dy: i32, map: &Map, objects: &mut [Object]) {
    let (x, y) = objects[id].pos();
    if !is_blocked(x + dx, y + dy, map, objects) {
        objects[id].set_pos(x + dx, y + dy);
    }
}

fn player_move_or_attack(dx: i32, dy: i32, map: &Map, objects: &mut [Object]) {
    // the coordinates the player is moving to/attacking
    let x = objects[PLAYER].x + dx;
    let y = objects[PLAYER].y + dy;

    // try to find an attackable object there
    let target_id = objects.iter().position(|object| object.pos() == (x, y));

    // attack if target found, move otherwise
    match target_id {
        Some(target_id) => {
            println!(
                "The {} laughs at your puny efforts to attack him!",
                objects[target_id].name
            );
        }
        None => {
            move_by(PLAYER, dx, dy, map, objects);
        }
    }
}

fn move_towards(id: usize, target_x: i32, target_y: i32, map: &Map, objects: &mut [Object]) {
    // vector from this object to the target, and distance
    let dx = target_x - objects[id].x;
    let dy = target_y - objects[id].y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    // normalize it to length 1 (preserving direction), then round it and
    // convert to integer so the movement is restricted to the map grid
    let dx = (dx as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;
    move_by(id, dx, dy, map, objects);
}
