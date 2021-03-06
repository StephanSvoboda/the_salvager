use specs::prelude::*;
use specs_derive::*;
use specs::saveload::{Marker, ConvertSaveload};
use specs::error::NoError;
use serde::{Serialize, Deserialize};
use rltk::{RGB};

#[derive(Component, ConvertSaveload, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order : i32
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles : Vec<rltk::Point>,
    pub range : i32,
    pub dirty : bool
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Robot {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Name {
    pub name : String
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct CombatStats {
    pub hp : Pool,
    pub energy : Pool,
    pub oxygen : Pool,
    pub defense : i32,
    pub power : i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToMelee {
    pub target : Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct SufferDamage {
    pub amount : Vec<i32>
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount : vec![amount] };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct DrainEnergy {
    pub amount : Vec<i32>
}

impl DrainEnergy {
    pub fn new_energy(store: &mut WriteStorage<DrainEnergy>, victim: Entity, amount: i32) {
        if let Some(draining) = store.get_mut(victim) {
            draining.amount.push(amount);
        } else {
            let dmg = DrainEnergy { amount : vec![amount] };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct BreathOxygen {
    pub amount : Vec<i32>
}

impl BreathOxygen {
    pub fn new_breath(store: &mut WriteStorage<BreathOxygen>, victim: Entity, amount: i32) {
        if let Some(breathing) = store.get_mut(victim) {
            breathing.amount.push(amount);
        } else {
            let breath = BreathOxygen { amount : vec![amount] };
            store.insert(victim, breath).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Item {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub heal_amount : i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InBackpack {
    pub owner : Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToPickupItem {
    pub collected_by : Entity,
    pub item : Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToUseItem {
    pub item : Entity,
    pub target: Option<rltk::Point>
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToDropItem {
    pub item : Entity
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Consumable {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Ranged {
    pub range : i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub damage : i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
    pub radius : i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Confusion {
    pub turns : i32
}

pub struct SerializeMe;

// Special component that exists to help serialize the game data
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map : super::map::Map
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pool {
    pub max: i32,
    pub current: i32,
    pub name: String
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot { Weapon }

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable {
    pub slot : EquipmentSlot
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Equipped {
    pub owner : Entity,
    pub slot : EquipmentSlot
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct MeleePowerBonus {
    pub power : i32
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct RangedWeapon {
    pub range: i32,
    pub damage: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToRemoveItem {
    pub item : Entity
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Target {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToShoot {
    pub target : Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct ProvidesOxygen {
    pub oxygen_amount : i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct ProvidesEnergy {
    pub energy_amount : i32
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ArtefactFromYendoria {}