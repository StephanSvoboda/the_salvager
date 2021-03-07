use rltk::Point;
use rltk::{Rltk, GameState, RGB};
use specs::prelude::*;

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod robot_ai_system;
use robot_ai_system::RobotAI;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {Paused, Running}

pub struct State {
    pub ecs: World,
    pub run_state: RunState
}


impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = RobotAI{};
        mob.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk){
        ctx.cls();

        if self.run_state == RunState::Running {
            self.run_systems();
            self.run_state = RunState::Paused;
        }else{
            self.run_state = player_input(self, ctx);
        }

        draw_map(&mut self.ecs, ctx);
        
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();
        
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)}
        }
    }
}

fn main() -> rltk::BError{
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
        let mut gs = State{
            ecs: World::new(),
            run_state : RunState::Running
        };
        
        //register Components
        gs.ecs.register::<Position>();
        gs.ecs.register::<Renderable>();
        gs.ecs.register::<Player>();
        gs.ecs.register::<Viewshed>();
        gs.ecs.register::<Robot>();
        gs.ecs.register::<Name>();        
        
        //create map
        let map : Map = Map::new_map_rooms_and_corridors();
        let (player_x, player_y) = map.rooms[0].center();
        let mut rng = rltk::RandomNumberGenerator::new();
        for (i, room) in map.rooms.iter().skip(1).enumerate() {
            let (x,y) = room.center();

            let glyph : rltk::FontCharType;
            let name : String;
            let roll = rng.roll_dice(1, 2);
            match roll {
                1 => { glyph = rltk::to_cp437('R'); name = "Robot".to_string();}
                _ => { glyph = rltk::to_cp437('m'); name = "Miningrobot".to_string();}
            }
            gs.ecs.create_entity()
                .with(Position{x,y})
                .with(Renderable{
                    glyph: glyph,
                    fg: RGB::named(rltk::BLUE),
                    bg: RGB::named(rltk::BLACK)
                })
                .with(Viewshed{ 
                    visible_tiles: Vec::new(), 
                    range: 8, 
                    dirty: true})
                .with(Robot{})
                .with(Name{name: format!("{} #{}", &name, i)})
                .build();
        }
        gs.ecs.insert(map);
        
        //add player
        gs.ecs
            .create_entity()
            .with(Position {x: player_x, y: player_y})
            .with(Renderable{
                glyph: rltk::to_cp437('@'),
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Player{})
            .with(Viewshed{ visible_tiles: Vec::new(), range : 8, dirty: true})
            .with(Name {name: "Player".to_string()})
            .build();
        gs.ecs.insert(Point::new(player_x, player_y));

    rltk::main_loop(context, gs)
}
