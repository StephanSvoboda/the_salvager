use specs::prelude::*;
use super::{Viewshed, Robot, Name, Map, Position};
use rltk::{Point, console};

pub struct RobotAI {}

impl <'a> System<'a> for RobotAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Robot>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_position, mut viewsheds, robots, names, mut positions) = data;

        for (_viewshed, _robot, _name,mut pos) in (&mut viewsheds, &robots, &names, &mut positions).join() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_position);
            if distance < 1.5 {
                // Attack goes here
                console::log(&format!("{} shouts insults", _name.name));
                return;
            }
            if _viewshed.visible_tiles.contains(&*player_position){
                console::log(format!("{} sees Player at {},{}",_name.name, player_position.x, player_position.y));
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32,
                    map.xy_idx(player_position.x, player_position.y) as i32,
                    &mut *map
                );
                if path.success && path.steps.len()>1 {
                    pos.x = path.steps[1] as i32 % map.width;
                    pos.y = path.steps[1] as i32 / map.width;
                    console::log(format!("{} hunts player moving to {},{}", _name.name, pos.x, pos.y));
                    _viewshed.dirty = true;
                }
            }
        }
    }
}