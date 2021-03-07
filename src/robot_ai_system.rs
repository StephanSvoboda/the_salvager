use specs::prelude::*;
use super::{Viewshed, Robot, Name};
use rltk::{field_of_view, Point, console};

pub struct RobotAI {}

impl <'a> System<'a> for RobotAI {
    type SystemData = (
        ReadExpect<'a, Point>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Robot>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_position, viewsheds, robots, names) = data;

        for (_viewshed, _robot, _name) in (&viewsheds, &robots, &names).join() {
            if _viewshed.visible_tiles.contains(&*player_position){
                console::log(format!("{} sees Player at {},{}",_name.name, player_position.x, player_position.y));
            }
        }
    }
}