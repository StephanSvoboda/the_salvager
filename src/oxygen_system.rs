use super::{gamelog::GameLog, CombatStats, Name, Player, RunState, SufferDamage};
use crate::BreathOxygen;
use specs::prelude::*;
use std::cmp::max;

pub struct OxygenSystem {}

impl<'a> System<'a> for OxygenSystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, BreathOxygen>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut log, mut stats, mut breaths, mut damages) = data;

        for (entity, mut stats, breath) in (&entities, &mut stats, &breaths).join() {
            stats.oxygen.current = max(0, stats.oxygen.current - breath.amount.iter().sum::<i32>());
            match stats.oxygen.current {
                0 => {
                    SufferDamage::new_damage(&mut damages, entity, 1);
                    log.entries.push(format!("You suffer damage because you are out of air."))
                },
                25 => log.entries.push(format!("Oxygen level dropped to 25%")),
                _ => {}
            }
        }

        breaths.clear();
    }
}
