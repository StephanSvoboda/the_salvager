use specs::prelude::*;
use super::{Viewshed, Position, Robot};
use rltk::{field_of_view, Point, console};

pub struct RobotAI {}

impl <'a> System<'a> for RobotAI {
    type SystemData = (
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Robot>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (viewshed, position, robot) = data;

        for (_viewshed, _position, _robot) in (&viewshed, &position, &robot).join() {
            console::log("I am Robot");
        }
    }
}