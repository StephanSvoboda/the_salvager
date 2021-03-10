use super::{gamelog::GameLog, CombatStats, Name, SufferDamage};
use crate::{Equipped, RangedWeapon, WantsToShoot};
use specs::prelude::*;

pub struct RangedCombatSystem {}

impl<'a> System<'a> for RangedCombatSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToShoot>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, RangedWeapon>,
                        ReadStorage<'a, Equipped>
    );

    fn run(&mut self, data : Self::SystemData) {
        let (entities, mut log, mut wants_to_shoot, names, combat_stats, mut inflict_damage, ranged_weapons, equipped) = data;

        for (entity, wants_to_shoot, name, stats) in (&entities, &wants_to_shoot, &names, &combat_stats).join() {
            if stats.hp.current > 0 {
                let mut range_power = 0;
                for (_item_entity, ranged_weapon, equipped_by) in (&entities, &ranged_weapons, &equipped).join() {
                    if equipped_by.owner == entity {
                        range_power = ranged_weapon.damage;
                    }
                }

                let target_stats = combat_stats.get(wants_to_shoot.target).unwrap();
                if target_stats.hp.current > 0 {
                    let target_name = names.get(wants_to_shoot.target).unwrap();

                    let damage = i32::max(0, range_power - target_stats.defense );


                    if damage == 0 {
                        log.entries.push(format!(
                            "{} is unable to hurt {}",
                            &name.name, &target_name.name
                        ));
                    } else {
                        log.entries.push(format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, damage
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, wants_to_shoot.target, damage);
                    }
                } else {}
            }
        }

        wants_to_shoot.clear();
    }
}
