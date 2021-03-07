use super::WantsToInjectStimPack;
use super::StimPack;
use super::CombatStats;
use specs::prelude::*;
use super::{WantsToPickupItem, Name, InBackpack, Position, gamelog::GameLog};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack{ owner: pickup.collected_by }).expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!("You pick up the {}.", names.get(pickup.item).unwrap().name));
            }
        }

        wants_pickup.clear();
    }
}

pub struct StimPackUseSystem {}

impl<'a> System<'a> for StimPackUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToInjectStimPack>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, StimPack>,
                        WriteStorage<'a, CombatStats>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_injection, names, stim_packs, mut combat_stats) = data;

        for (entity, inject, stats) in (&entities, &wants_injection, &mut combat_stats).join() {
            let stim_pack = stim_packs.get(inject.stim_pack);
            match stim_pack {
                None => {
                }
                Some(stim_pack) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + stim_pack.heal_amount);
                    if entity == *player_entity {
                        gamelog.entries.push(format!("You inject the {}, healing {} hp.", names.get(inject.stim_pack).unwrap().name, stim_pack.heal_amount));
                    }
                    entities.delete(inject.stim_pack).expect("Delete failed");
                }
            }
        }

        wants_injection.clear();
    }
}