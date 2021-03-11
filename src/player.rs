use rltk::{Point};
use rltk::{VirtualKeyCode, Rltk};
use specs::prelude::*;
use super::{Position, Player, Map, State, Viewshed, RunState, CombatStats, WantsToMelee, Item, gamelog::GameLog, WantsToPickupItem};
use std::cmp::{min, max};
use crate::{Equipped, RangedWeapon, Robot, Target, WantsToShoot, Name, BreathOxygen, ArtefactFromYendoria};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewshed = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, _viewshed) in (&entities, &mut players, &mut positions, &mut viewshed).join() {
        if pos.x + delta_x < 1 || pos.x + delta_x > map.width-1 || pos.y + delta_y < 1 || pos.y + delta_y > map.height-1 { return; }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                wants_to_melee.insert(entity, WantsToMelee{ target: *potential_target }).expect("Add target failed");
                return;
            }
        }
        if !map.blocked_tiles[destination_idx] {
            pos.x = min(map.width-1 , max(0, pos.x + delta_x));
            pos.y = min(map.height-1, max(0, pos.y + delta_y));

            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;

            _viewshed.dirty = true;
        }
    }
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item : Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(*player_entity, WantsToPickupItem{ collected_by: *player_entity, item }).expect("Unable to insert want to pickup");
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState{
    // Hotkeys
    if ctx.shift && ctx.key.is_some() {
        let key : Option<i32> =
            match ctx.key.unwrap() {
                VirtualKeyCode::Key1 => Some(1),
                VirtualKeyCode::Key2 => Some(2),
                VirtualKeyCode::Key3 => Some(3),
                VirtualKeyCode::Key4 => Some(4),
                VirtualKeyCode::Key5 => Some(5),
                VirtualKeyCode::Key6 => Some(6),
                VirtualKeyCode::Key7 => Some(7),
                VirtualKeyCode::Key8 => Some(8),
                VirtualKeyCode::Key9 => Some(9),
                _ => None
            };
        if let Some(key) = key {
            return use_consumable_hotkey(gs, key-1);
        }
    }
    // Player movement
    match ctx.key {
        None => { return RunState::AwaitingInput} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left |
            VirtualKeyCode::Numpad4 |
            VirtualKeyCode::H => try_move_player(-1, 0, &mut gs.ecs),

            VirtualKeyCode::Right |
            VirtualKeyCode::Numpad6 |
            VirtualKeyCode::L => try_move_player(1, 0, &mut gs.ecs),

            VirtualKeyCode::Up |
            VirtualKeyCode::Numpad8 |
            VirtualKeyCode::K => try_move_player(0, -1, &mut gs.ecs),

            VirtualKeyCode::Down |
            VirtualKeyCode::Numpad2 |
            VirtualKeyCode::J => try_move_player(0, 1, &mut gs.ecs),

            // Diagonals
            VirtualKeyCode::Numpad9 |
            VirtualKeyCode::Y => try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad7 |
            VirtualKeyCode::U => try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad3 |
            VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad1 |
            VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),

            VirtualKeyCode::G => {
                get_item(&mut gs.ecs);
                if check_game_won(&mut gs.ecs) {
                    return RunState::GameWon;
                }
            },
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::D => return RunState::ShowDropItem,
            VirtualKeyCode::Escape => return RunState::SaveGame,
            VirtualKeyCode::R => return RunState::ShowRemoveItem,
            // Ranged
            VirtualKeyCode::V => {
                cycle_target(&mut gs.ecs);
                return RunState::AwaitingInput;
            }
            VirtualKeyCode::F => fire_on_target(&mut gs.ecs),

            _ => { return RunState::AwaitingInput}
        },
    }
    RunState::PlayerTurn
}

fn check_game_won(ecs: &mut World) -> bool {
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let wants_pick_ups = ecs.read_storage::<WantsToPickupItem>();

    for (wants_pick_up,entity) in (&wants_pick_ups, &entities).join() {
        if wants_pick_up.collected_by == * player_entity {
            if let Some(artefact) = ecs.read_storage::<ArtefactFromYendoria>().get(wants_pick_up.item){
                return true;
            }
        }
    }
    false
}

fn use_consumable_hotkey(gs: &mut State, key: i32) -> RunState {
    use super::{Consumable, InBackpack, WantsToUseItem};

    let consumables = gs.ecs.read_storage::<Consumable>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let player_entity = gs.ecs.fetch::<Entity>();
    let entities = gs.ecs.entities();
    let mut carried_consumables = Vec::new();
    for (entity, carried_by, _consumable) in (&entities, &backpack, &consumables).join() {
        if carried_by.owner == *player_entity {
            carried_consumables.push(entity);
        }
    }

    if (key as usize) < carried_consumables.len() {
        use crate::components::Ranged;
        if let Some(ranged) = gs.ecs.read_storage::<Ranged>().get(carried_consumables[key as usize]) {
            return RunState::ShowTargeting{ range: ranged.range, item: carried_consumables[key as usize] };
        }
        let mut intent = gs.ecs.write_storage::<WantsToUseItem>();
        intent.insert(
            *player_entity,
            WantsToUseItem{ item: carried_consumables[key as usize], target: None }
        ).expect("Unable to insert intent");
        return RunState::PlayerTurn;
    }
    RunState::PlayerTurn
}

fn get_player_target_list(ecs : &mut World) -> Vec<(f32,Entity)> {
    let mut possible_targets : Vec<(f32,Entity)> = Vec::new();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let player_entity = ecs.fetch::<Entity>();
    let equipped = ecs.read_storage::<Equipped>();
    let ranged_weapons = ecs.read_storage::<RangedWeapon>();
    let map = ecs.fetch::<Map>();
    let positions = ecs.read_storage::<Position>();
    let robots = ecs.read_storage::<Robot>();
    for (equipped, ranged_weapon) in (&equipped, &ranged_weapons).join() {
        if equipped.owner == *player_entity {
            let range = ranged_weapon.range;

            if let Some(viewshed) = viewsheds.get(*player_entity) {
                let player_pos = positions.get(*player_entity).unwrap();
                for tile_point in viewshed.visible_tiles.iter() {
                    let tile_idx = map.xy_idx(tile_point.x, tile_point.y);
                    let distance_to_target = rltk::DistanceAlg::Pythagoras.distance2d(*tile_point, rltk::Point::new(player_pos.x, player_pos.y));
                    if distance_to_target < range as f32 {
                        for possible_target in map.tile_content[tile_idx].iter() {
                            if *possible_target != *player_entity && robots.get(*possible_target).is_some() {
                                possible_targets.push((distance_to_target, *possible_target));
                            }
                        }
                    }
                }
            }
        }
    }

    possible_targets.sort_by(|a,b| a.0.partial_cmp(&b.0).unwrap());
    possible_targets
}

pub fn end_turn_targeting(ecs: &mut World) {
    let possible_targets = get_player_target_list(ecs);
    let mut targets = ecs.write_storage::<Target>();
    let entities = ecs.entities();
    let mut last_target : Option<Entity> = None;
    for (entity, _target) in (&entities, &targets).join() {
        last_target = Some(entity)
    }
    targets.clear();
    if !possible_targets.is_empty() {
        let mut last_target_inserted = false;
        if let Some(last_target) = last_target {
            for entity in &possible_targets {
                if entity.1 == last_target {
                    targets.insert(entity.1, Target {}).expect("Insert fail");
                    last_target_inserted = true;
                }
            }
            if !last_target_inserted {
                targets.insert(possible_targets[0].1, Target {}).expect("Insert fail");
            }
        }else{
            targets.insert(possible_targets[0].1, Target {}).expect("Insert fail");
        }
    }
}

fn cycle_target(ecs: &mut World) {
    let possible_targets = get_player_target_list(ecs);
    let mut targets = ecs.write_storage::<Target>();
    let entities = ecs.entities();
    let mut current_target : Option<Entity> = None;

    for (e,_t) in (&entities, &targets).join() {
        current_target = Some(e);
    }

    targets.clear();
    if let Some(current_target) = current_target {
        if possible_targets.len() > 1 {
            let mut index = 0;
            for (i, target) in possible_targets.iter().enumerate() {
                if target.1 == current_target {
                    index = i;
                }
            }
            let mut next_index = 0;
            if index + 1 != possible_targets.len(){
                next_index = index + 1;
            }
            targets.insert(possible_targets[next_index].1, Target{});
        }else{
            targets.insert(possible_targets[0].1, Target{});
        }
    }else{
        if possible_targets.len() > 0 {
            targets.insert(possible_targets[0].1, Target{});
        }
    }
}

fn fire_on_target(ecs: &mut World) {
    let targets = ecs.write_storage::<Target>();
    let entities = ecs.entities();
    let mut current_target : Option<Entity> = None;
    let mut log = ecs.fetch_mut::<GameLog>();


    for (e,_t) in (&entities, &targets).join() {
        current_target = Some(e);
    }

    if let Some(target) = current_target {
        let player_entity = ecs.fetch::<Entity>();
        let mut shoot_store = ecs.write_storage::<WantsToShoot>();
        let names = ecs.read_storage::<Name>();
        if let Some(name) = names.get(target) {
            log.entries.push(format!("You fire at {}", name.name));
        }
        shoot_store.insert(*player_entity, WantsToShoot{ target }).expect("Insert Fail");
    } else {
        log.entries.push("You don't have a target selected!".to_string());
    }

}

pub fn end_turn_breathing(ecs: &mut World){
    let player_entity = ecs.fetch::<Entity>();
    let mut oxygen_store = ecs.write_storage::<BreathOxygen>();
    BreathOxygen::new_breath(&mut oxygen_store, *player_entity, 1)
}