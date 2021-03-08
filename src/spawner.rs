use rltk::{ RGB, RandomNumberGenerator };
use specs::prelude::*;
use super::{
    CombatStats, 
    Player,
    Renderable, 
    Name, 
    Position, 
    Viewshed, 
    Robot, 
    BlocksTile, 
    map::MAP_WIDTH, 
    Rect, 
    ProvidesHealing, 
    Item,
    Consumable,
    Ranged,
    InflictsDamage,
    AreaOfEffect
};

const MAX_ROBOTS : i32 = 4;
const MAX_ITEMS : i32 = 2;

pub fn spawn_room(ecs: &mut World, room : &Rect) {
    let mut robots_spawn_points : Vec<usize> = Vec::new();
    let mut item_spawn_points : Vec<usize> = Vec::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        
        let num_robots = rng.roll_dice(1, MAX_ROBOTS + 2) - 3;
        for _i in 0 .. num_robots {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAP_WIDTH) + x;
                if !robots_spawn_points.contains(&idx) {
                    robots_spawn_points.push(idx);
                    added = true;
                }
            }
        }

        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;
        for _i in 0 .. num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAP_WIDTH) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }

    // Actually spawn the robots
    for idx in robots_spawn_points.iter() {
        let x = *idx % MAP_WIDTH;
        let y = *idx / MAP_WIDTH;
        random_robot(ecs, x as i32, y as i32);
    }

    // Actually spawn the stim packs
    for idx in item_spawn_points.iter() {
        let x = *idx % MAP_WIDTH;
        let y = *idx / MAP_WIDTH;
        random_item(ecs, x as i32, y as i32);
    }
}

pub fn player(ecs : &mut World, player_x : i32, player_y : i32) -> Entity {
    ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0
        })
        .with(Player{})
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Name{name: "Player".to_string() })
        .with(CombatStats{ max_hp: 30, hp: 30, defense: 2, power: 5 })
        .build()
}

pub fn random_robot(ecs: &mut World, x: i32, y: i32) {
    let roll :i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => { robot(ecs, x, y) }
        _ => { minin_robot(ecs, x, y) }
    }
}

fn robot(ecs: &mut World, x: i32, y: i32) {
    mob(ecs, x, y, rltk::to_cp437('R'), "Robot"); 
}

fn minin_robot(ecs: &mut World, x: i32, y: i32) {
    mob(ecs, x, y, rltk::to_cp437('m'), "Miningrobot");
}

fn mob<S : ToString>(ecs: &mut World, x: i32, y: i32, glyph : rltk::FontCharType, name : S) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph,
            fg: RGB::named(rltk::BLUE),
            bg: RGB::named(rltk::BLACK),
            render_order: 1
        })
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Robot{})
        .with(Name{ name : name.to_string() })
        .with(BlocksTile{})
        .with(CombatStats{ max_hp: 16, hp: 16, defense: 1, power: 4 })
        .build();
}

fn random_item(ecs: &mut World, x: i32, y: i32) {
    let roll :i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 3);
    }
    match roll {
        1 => { stim_packs(ecs, x, y) }
        2 => { laser_torch(ecs, x, y) }
        _ => { grenades(ecs, x, y) }
    }
}

fn stim_packs(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Basic Stim Pack".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(ProvidesHealing{ heal_amount: 8 })
        .build();
}

fn grenades(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('g'),
            fg: RGB::named(rltk::DARK_OLIVE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Grenade".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(InflictsDamage{ damage: 8 })
        .with(AreaOfEffect{ radius: 3 })
        .build();
}

fn laser_torch(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{
            glyph: rltk::to_cp437('t'),
            fg: RGB::named(rltk::LIGHTYELLOW1),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Laser torch".to_string() })
        .with(Item{})
        .with(Ranged{ range: 3 })
        .with(InflictsDamage{ damage: 4 })
        .build();
}