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
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
mod damage_system;
use damage_system::DamageSystem;
mod gui;
mod gamelog;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { AwaitingInput, PreRun, PlayerTurn, MonsterTurn }

pub struct State {
    pub ecs: World,
}


impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = RobotAI{};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk){
        ctx.cls();
        let mut new_run_state;
        {
            let runstate = self.ecs.fetch::<RunState>();
            new_run_state = *runstate
        }

        match new_run_state {
            RunState::PreRun => {
                self.run_systems();
                new_run_state = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_run_state = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                new_run_state = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                new_run_state = RunState::AwaitingInput;
            }
        }
        
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_run_state;
        }
        damage_system::delete_the_dead(&mut self.ecs);
        
        draw_map(&mut self.ecs, ctx);
        
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();
        
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)}
        }
        gui::draw_ui(&self.ecs, ctx);
    }
}

fn main() -> rltk::BError{
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
        let mut gs = State{
            ecs: World::new()
        };
        
        //register Components
        gs.ecs.register::<Position>();
        gs.ecs.register::<Renderable>();
        gs.ecs.register::<Player>();
        gs.ecs.register::<Viewshed>();
        gs.ecs.register::<Robot>();
        gs.ecs.register::<Name>();
        gs.ecs.register::<BlocksTile>();
        gs.ecs.register::<CombatStats>();
        gs.ecs.register::<WantsToMelee>();
        gs.ecs.register::<SufferDamage>();
        
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
                .with(BlocksTile{})
                .with(CombatStats{ max_hp: 16, hp: 16, defense: 1, power: 4 })
                .build();
        }
        gs.ecs.insert(map);
        
        //add player
        let player_entity = gs.ecs
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
            .with(CombatStats{ max_hp: 30, hp: 30, defense: 2, power: 5 })
            .build();
        gs.ecs.insert(Point::new(player_x, player_y));
        gs.ecs.insert(player_entity);
        gs.ecs.insert(RunState::PreRun);
        gs.ecs.insert(gamelog::GameLog{ entries : vec!["The salvager take a deep breath.".to_string()] });

    rltk::main_loop(context, gs)
}
