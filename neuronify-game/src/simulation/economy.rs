use hecs::Entity;

use crate::components::{GlialCell, Ownership, PlayerId, PlayerEconomy};
use crate::constants::GLIAL_BLOCK_TRANSFER_RATE;

/// Refund building blocks into a player's economy (capped at max).
pub fn add_blocks(economy: &mut PlayerEconomy, amount: f64) {
    economy.building_blocks = (economy.building_blocks + amount).min(economy.max_building_blocks);
}

/// Try to deduct building blocks from a player's economy. Returns true if successful.
pub fn try_spend_blocks(economy: &mut PlayerEconomy, cost: f64) -> bool {
    if economy.building_blocks >= cost {
        economy.building_blocks -= cost;
        true
    } else {
        false
    }
}

/// Glial cells transfer stored building blocks into the player economy.
pub fn glial_contribute_blocks(world: &mut hecs::World, dt: f64, economy: &mut PlayerEconomy) {
    let glial_entities: Vec<Entity> = world
        .query::<(&GlialCell, &Ownership)>()
        .iter()
        .filter(|(_, (_, o))| o.player == PlayerId::Player1)
        .map(|(e, _)| e)
        .collect();

    for entity in glial_entities {
        let room = economy.max_building_blocks - economy.building_blocks;
        if room <= 0.0 {
            break;
        }
        if let Ok(mut glial) = world.get::<&mut GlialCell>(entity) {
            let transfer = (GLIAL_BLOCK_TRANSFER_RATE * dt)
                .min(glial.blocks_stored)
                .min(room);
            if transfer > 0.0 {
                glial.blocks_stored -= transfer;
                economy.building_blocks += transfer;
            }
        }
    }
}

