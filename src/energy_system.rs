use specs::prelude::*;
use super::{CombatStats, SufferDamage, Player, gamelog::GameLog, Name, RunState};
use crate::DrainEnergy;

pub struct EnergySystem {}

impl<'a> System<'a> for EnergySystem {
    type SystemData = ( WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, DrainEnergy> );

    fn run(&mut self, data : Self::SystemData) {
        let (mut stats, mut energies) = data;

        for (mut stats, energy) in (&mut stats, &energies).join() {
            stats.energy.current -= energy.amount.iter().sum::<i32>();
        }

        energies.clear();
    }

}