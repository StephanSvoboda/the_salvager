use specs::prelude::*;
use super::{Viewshed, Robot};
use rltk::{field_of_view, Point, console};

pub struct RobotAI {}

impl <'a> System<'a> for RobotAI {
    type SystemData = (
        ReadExpect<'a, Point>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Robot>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_position, viewsheds, robots) = data;

        for (_viewshed, _robot) in (&viewsheds, &robots).join() {
            if _viewshed.visible_tiles.contains(&*player_position){
                console::log(format!("Robot sees Player at {},{}", player_position.x, player_position.y));
            }
        }
    }
}