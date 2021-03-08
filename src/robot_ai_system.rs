use specs::prelude::*;
use super::{Viewshed, Robot, Map, Position, WantsToMelee, RunState, Confusion};
use rltk::{Point};

pub struct RobotAI {}

impl <'a> System<'a> for RobotAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Robot>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, Confusion>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (
            mut map, 
            player_pos, 
            player_entity, 
            _runstate, 
            entities, 
            mut viewshed, 
            robots, 
            mut position, 
            mut wants_to_melee, 
            mut confused
        ) = data;

        if *_runstate != RunState::MonsterTurn { return }

        for (entity, mut viewshed,_robot,mut pos) in (&entities, &mut viewshed, &robots, &mut position).join() {
            let mut can_act = true;

            let is_confused = confused.get_mut(entity);
            if let Some(i_am_confused) = is_confused{
                i_am_confused.turns -= 1;
                if i_am_confused.turns < 1 {
                    confused.remove(entity);
                }
                can_act = false;
            }

            if can_act {
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance < 1.5 {
                    wants_to_melee.insert(entity, WantsToMelee{ target: *player_entity }).expect("Unable to insert attack");
                }
                else if viewshed.visible_tiles.contains(&*player_pos) {
                    // Path to the player
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y),
                        map.xy_idx(player_pos.x, player_pos.y),
                        &mut *map
                    );
                    if path.success && path.steps.len()>1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked_tiles[idx] = false;
                        pos.x = path.steps[1] as i32 % map.width;
                        pos.y = path.steps[1] as i32 / map.width;
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked_tiles[idx] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        }
    }
}