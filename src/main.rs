use rltk::Point;
use rltk::{Rltk, GameState};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
extern crate serde;

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
use gui::MainMenuSelection;
mod gamelog;
mod spawner;
mod inventory_system;
use inventory_system::ItemCollectionSystem;
use inventory_system::ItemUseSystem;
use inventory_system::ItemDropSystem;
use crate::inventory_system::ItemRemoveSystem;
mod ranged_combat_system;
use ranged_combat_system::RangedCombatSystem;

mod saveload_system;
mod camera;
mod energy_system;
use energy_system::EnergySystem;
mod oxygen_system;
use oxygen_system::OxygenSystem;
use crate::gamelog::GameLog;


#[derive(PartialEq, Copy, Clone)]
pub enum RunState { 
    AwaitingInput,
    PreRun, 
    PlayerTurn, 
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range : i32, item : Entity} ,
    MainMenu { menu_selection : gui::MainMenuSelection },
    SaveGame,
    GameOver,
    ShowRemoveItem,
    GameWon
 }

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
        let mut ranged_combat_system = RangedCombatSystem{};
        ranged_combat_system.run_now(&self.ecs);
        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);
        let mut energy = EnergySystem{};
        energy.run_now(&self.ecs);
        let mut oxygen = OxygenSystem{};
        oxygen.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem{};
        pickup.run_now(&self.ecs);
        let mut item_use = ItemUseSystem{};
        item_use.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem{};
        drop_items.run_now(&self.ecs);
        let mut item_remove = ItemRemoveSystem{};
        item_remove.run_now(&self.ecs);
        self.ecs.maintain();

    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk){
        let mut new_run_state;
        {
            let runstate = self.ecs.fetch::<RunState>();
            new_run_state = *runstate
        }

        ctx.cls();

        match new_run_state {
            RunState::MainMenu{..} => {}
            _ => {

                camera::render_camera(&self.ecs, ctx);
                gui::draw_ui(&self.ecs, ctx);

            }
        }

        match new_run_state {
            RunState::MainMenu{ .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection{ selected } => new_run_state = RunState::MainMenu{ menu_selection: selected },
                    gui::MainMenuResult::Selected{ selected } => {
                        match selected {
                            gui::MainMenuSelection::NewGame => {
                                self.game_over_cleanup();
                                new_run_state = RunState::PreRun;
                            }
                            gui::MainMenuSelection::LoadGame => {
                                saveload_system::load_game(&mut self.ecs);
                                new_run_state = RunState::AwaitingInput;
                                saveload_system::delete_save();
                            }
                            gui::MainMenuSelection::Quit => { ::std::process::exit(0); }
                        }
                    }
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                new_run_state = RunState::MainMenu{ menu_selection : gui::MainMenuSelection::LoadGame };
            }
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                new_run_state = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_run_state = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                player::end_turn_breathing(&mut self.ecs);
                self.run_systems();
                player::end_turn_targeting(&mut self.ecs);
                self.ecs.maintain();
                new_run_state = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                new_run_state = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            new_run_state = RunState::ShowTargeting{ range: is_item_ranged.range, item: item_entity };
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem{ item: item_entity, target: None }).expect("Unable to insert intent");
                            new_run_state = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowRemoveItem => {
                let result = gui::remove_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToRemoveItem{ item: item_entity }).expect("Unable to insert intent");
                        new_run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem{ item: item_entity }).expect("Unable to insert intent");
                        new_run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowTargeting{range, item} => {
                let result = gui::ranged_target(self, ctx, range);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem{ item, target: result.1 }).expect("Unable to insert intent");
                        new_run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameEndResult::NoSelection => {}
                    gui::GameEndResult::QuitToMenu => {
                        self.game_over_cleanup();
                        new_run_state = RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame };
                    }
                }
            }
            RunState::GameWon => {
                let result = gui::game_won(ctx);
                match result {
                    gui::GameEndResult::NoSelection => {}
                    gui::GameEndResult::QuitToMenu => {
                        self.game_over_cleanup();
                        new_run_state = RunState::MainMenu{ menu_selection: gui::MainMenuSelection::NewGame };
                    }
                }
            }
        }
        
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_run_state;
        }
        damage_system::delete_the_dead(&mut self.ecs);
        
        
    }
}

impl State {
    fn game_over_cleanup(&mut self) {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }
    
        // Build a new map and place the player
        let map;
        {
            let mut map_ressource = self.ecs.write_resource::<Map>();
            *map_ressource = Map::new_map_rooms_and_corridors();
            map = map_ressource.clone();
        }
    
        // Spawn bad guys
        for room in map.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room);
        }
    
        // Place the player and update resources
        let (player_x, player_y) = map.rooms[0].center();
        let player_entity = spawner::player(&mut self.ecs, player_x, player_y);
        self.spawn_start_inventory(player_x, player_y);
        self.spawn_artefact_of_yendoria(map);
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(player_x, player_y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let mut player_entity_writer = self.ecs.write_resource::<Entity>();
        *player_entity_writer = player_entity;
        let player_pos_comp = position_components.get_mut(player_entity);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        // Mark the player's visibility as dirty
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        let vs = viewshed_components.get_mut(player_entity);
        if let Some(vs) = vs {
            vs.dirty = true;
        }

        let mut log = self.ecs.fetch_mut::<GameLog>();
        log.entries = vec!["Salvager retrieve another Artefact of Yendoria.".to_string()]
    }

    fn spawn_start_inventory(&mut self, player_x: i32, player_y: i32) {
        spawner::blaster(&mut self.ecs, player_x + 1, player_y);
        spawner::battery(&mut self.ecs, player_x + 2, player_y);
        spawner::stim_packs(&mut self.ecs, player_x + 2, player_y + 1);
        spawner::oxygen_tank(&mut self.ecs, player_x + 2, player_y + 2);
    }

    fn spawn_artefact_of_yendoria(&mut self, map: Map) {
        let (artefact_x, artefact_y) = map.rooms.last().unwrap().center();
        spawner::artefact(&mut self.ecs, artefact_x, artefact_y);
    }
}

fn main() -> rltk::BError{
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple(80,60)
        .unwrap()
        .with_title("The Salvager - 7DLR 2021")
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
    gs.ecs.register::<Item>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<MeleePowerBonus>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<RangedWeapon>();
    gs.ecs.register::<Target>();
    gs.ecs.register::<WantsToShoot>();
    gs.ecs.register::<DrainEnergy>();
    gs.ecs.register::<BreathOxygen>();
    gs.ecs.register::<ProvidesOxygen>();
    gs.ecs.register::<ProvidesEnergy>();
    gs.ecs.register::<ArtefactFromYendoria>();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
    
    let map : Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);


    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    spawner::blaster(&mut gs.ecs, player_x + 1, player_y);
    spawner::battery(&mut gs.ecs, player_x + 2, player_y);
    spawner::stim_packs(&mut gs.ecs, player_x + 2, player_y + 1);
    spawner::oxygen_tank(&mut gs.ecs, player_x + 2, player_y + 2);
    let (artefact_x, artefact_y) = map.rooms.last().unwrap().center();
    spawner::artefact(&mut gs.ecs, artefact_x, artefact_y);

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::MainMenu{menu_selection: MainMenuSelection::NewGame });
    gs.ecs.insert(gamelog::GameLog{ entries : vec!["Salvager retrieve the Artefact of Yendoria.".to_string()] });

    rltk::main_loop(context, gs)
}
