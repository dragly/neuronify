//! Combat systems — one function per unit mechanic.
//!
//! Units fire visible projectiles toward their targets; damage is applied on
//! projectile arrival. Call order each frame:
//! move → fire (axon/engulf/burst) → advance_projectiles → absorption → despawn.

use glam::Vec3;
use neuronify_core::{Compartment, LeakyNeuron, OriginNeuron, GeneratorDynamics, Position};

use crate::components::{
    AttackProjectile, AxonCutter, AxonHealth, BurstAttack, Faction, GlialAbsorption, Health,
    MetabolicState, MobileUnit, NeuronEngulfment, Ownership, PlayerId,
};
use crate::constants::ATTACK_PROJECTILE_SPEED;

// ── Target finders ────────────────────────────────────────────────────────────

fn entity_pos(world: &hecs::World, entity: hecs::Entity) -> Option<Vec3> {
    world.get::<&Position>(entity).ok().map(|p| p.position)
}

fn is_alive(world: &hecs::World, entity: hecs::Entity) -> bool {
    world.contains(entity)
}

fn nearest_of(candidates: &[(hecs::Entity, Vec3)], from: Vec3) -> Option<(hecs::Entity, Vec3)> {
    candidates
        .iter()
        .min_by(|a, b| {
            a.1.distance(from)
                .partial_cmp(&b.1.distance(from))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
}

fn player_neuron_positions(world: &hecs::World) -> Vec<(hecs::Entity, Vec3)> {
    world
        .query::<(&Position, &LeakyNeuron, &Ownership)>()
        .with::<&MetabolicState>()
        .iter()
        .filter(|(_, (_, _, o))| o.player == PlayerId::Player1)
        .map(|(e, (p, _, _))| (e, p.position))
        .collect()
}

fn player_axon_positions(world: &hecs::World) -> Vec<(hecs::Entity, Vec3)> {
    world
        .query::<(&Position, &Compartment, &AxonHealth, &Ownership)>()
        .iter()
        .filter(|(_, (_, _, _, o))| o.player == PlayerId::Player1)
        .map(|(e, (p, _, _, _))| (e, p.position))
        .collect()
}

fn priority_neuron(world: &hecs::World, from: Vec3) -> Option<(hecs::Entity, Vec3)> {
    let origins: Vec<_> = world
        .query::<(&Position, &LeakyNeuron, &Ownership)>()
        .with::<&OriginNeuron>()
        .iter()
        .filter(|(_, (_, _, o))| o.player == PlayerId::Player1)
        .map(|(e, (p, _, _))| (e, p.position))
        .collect();
    if let Some(t) = nearest_of(&origins, from) {
        return Some(t);
    }
    let gens: Vec<_> = world
        .query::<(&Position, &GeneratorDynamics, &Ownership)>()
        .iter()
        .filter(|(_, (_, _, o))| o.player == PlayerId::Player1)
        .map(|(e, (p, _, _))| (e, p.position))
        .collect();
    if let Some(t) = nearest_of(&gens, from) {
        return Some(t);
    }
    nearest_of(&player_neuron_positions(world), from)
}

fn enemy_mobile_positions(world: &hecs::World, my_faction: Faction) -> Vec<(hecs::Entity, Vec3)> {
    world
        .query::<(&Position, &MobileUnit)>()
        .iter()
        .filter(|(_, (_, m))| m.faction != my_faction)
        .map(|(e, (p, _))| (e, p.position))
        .collect()
}

// ── 1. Move mobile units ──────────────────────────────────────────────────────

pub fn move_mobile_units(world: &mut hecs::World, dt: f32) {
    let axon_targets = player_axon_positions(world);
    let neuron_targets = player_neuron_positions(world);

    let mut moves: Vec<(hecs::Entity, Vec3)> = Vec::new();
    let mut target_updates: Vec<(hecs::Entity, Option<hecs::Entity>)> = Vec::new();

    for (entity, (pos, mobile)) in world.query::<(&Position, &MobileUnit)>().iter() {
        let from = pos.position;
        let faction = mobile.faction;

        let valid_target = mobile
            .target
            .filter(|&t| is_alive(world, t))
            .and_then(|t| entity_pos(world, t).map(|p| (t, p)));

        let target = if let Some(t) = valid_target {
            Some(t)
        } else {
            let has_axon_cutter = world.get::<&AxonCutter>(entity).is_ok();
            let has_engulfment = world.get::<&NeuronEngulfment>(entity).is_ok();
            let has_burst = world.get::<&BurstAttack>(entity).is_ok();

            if faction != Faction::Biological {
                if has_axon_cutter {
                    nearest_of(&axon_targets, from)
                } else if has_engulfment {
                    nearest_of(&neuron_targets, from)
                } else if has_burst {
                    priority_neuron(world, from)
                } else {
                    nearest_of(&neuron_targets, from)
                }
            } else {
                let enemies = enemy_mobile_positions(world, Faction::Biological);
                nearest_of(&enemies, from)
            }
        };

        if let Some((target_entity, target_pos)) = target {
            // NeuronEngulfment units halt just inside fire_range so projectiles
            // have a visible flight path.  Stopping at exactly fire_range risks
            // floating-point distance slightly above the threshold, so we park
            // 0.5 units inside the boundary to guarantee the fire check passes.
            let standoff = world
                .get::<&NeuronEngulfment>(entity)
                .map(|e| (e.fire_range - 0.5).max(0.0))
                .unwrap_or(0.0);
            let dist = from.distance(target_pos);
            if dist > standoff {
                let dir = (target_pos - from).normalize_or_zero();
                let step = mobile.speed * dt;
                // Don't overshoot the standoff ring.
                let new_pos = if step >= dist - standoff {
                    from + dir * (dist - standoff).max(0.0)
                } else {
                    from + dir * step
                };
                moves.push((entity, new_pos));
            }
            target_updates.push((entity, Some(target_entity)));
        } else {
            target_updates.push((entity, None));
        }
    }

    for (entity, new_pos) in moves {
        if let Ok(mut pos) = world.get::<&mut Position>(entity) {
            pos.position = new_pos;
        }
    }
    for (entity, new_target) in target_updates {
        if let Ok(mut mobile) = world.get::<&mut MobileUnit>(entity) {
            mobile.target = new_target;
        }
    }
}

// ── 2. Axon-cutting projectiles ───────────────────────────────────────────────

/// Each AxonCutter fires a cyan projectile at its movement target on cooldown.
pub fn apply_axon_cutting(world: &mut hecs::World, dt: f32) {
    // Snapshot current axon targets.
    let axon_targets = player_axon_positions(world);

    let cutters: Vec<(hecs::Entity, Vec3, f32, f32, f32, Option<hecs::Entity>)> = world
        .query::<(&Position, &AxonCutter, &MobileUnit)>()
        .iter()
        .map(|(e, (p, c, m))| {
            (e, p.position, c.shot_damage, c.shoot_cooldown, c.shoot_timer, m.target)
        })
        .collect();

    let mut to_spawn: Vec<(Vec3, hecs::Entity, f32, f32)> = Vec::new(); // pos, target, damage, speed

    for (entity, pos, damage, cooldown, timer, mobile_target) in cutters {
        let new_timer = timer - dt;
        if let Ok(mut c) = world.get::<&mut AxonCutter>(entity) {
            c.shoot_timer = new_timer;
        }

        if new_timer > 0.0 {
            continue;
        }

        // Target: use movement target if valid, else nearest axon.
        let target = mobile_target
            .filter(|&t| is_alive(world, t) && world.get::<&AxonHealth>(t).is_ok())
            .or_else(|| nearest_of(&axon_targets, pos).map(|(e, _)| e));

        let Some(target_entity) = target else { continue };

        if let Ok(mut c) = world.get::<&mut AxonCutter>(entity) {
            c.shoot_timer = cooldown;
        }
        to_spawn.push((pos, target_entity, damage, ATTACK_PROJECTILE_SPEED * 1.2));
    }

    for (pos, target, damage, speed) in to_spawn {
        world.spawn((
            Position { position: pos },
            AttackProjectile {
                target,
                speed,
                health_damage: 0.0,
                axon_damage: damage,
                // Bright cyan sparks — matches microglia color palette.
                color: glam::Vec3::new(0.0, 1.0, 0.92),
                radius: 0.45,
            },
        ));
    }
}

// ── 3. Neuron-engulfment projectiles ──────────────────────────────────────────

/// Each NeuronEngulfment unit fires a magenta projectile at its neuron target on cooldown.
pub fn apply_neuron_engulfment(world: &mut hecs::World, dt: f32) {
    let engulfers: Vec<(hecs::Entity, Vec3, f32, f32, f32, f32, Option<hecs::Entity>)> = world
        .query::<(&Position, &NeuronEngulfment)>()
        .iter()
        .map(|(e, (p, eng))| {
            (e, p.position, eng.shot_damage, eng.shoot_cooldown, eng.shoot_timer, eng.fire_range, eng.target)
        })
        .collect();

    let mut to_spawn: Vec<(Vec3, hecs::Entity, f32)> = Vec::new(); // pos, target, damage

    for (entity, pos, damage, cooldown, timer, fire_range, stored_target) in engulfers {
        let new_timer = timer - dt;
        if let Ok(mut eng) = world.get::<&mut NeuronEngulfment>(entity) {
            eng.shoot_timer = new_timer;
        }

        // Find/validate target.
        let target = stored_target
            .filter(|&t| is_alive(world, t))
            .or_else(|| nearest_of(&player_neuron_positions(world), pos).map(|(e, _)| e));
        let Some(target_entity) = target else { continue };

        if let Ok(mut eng) = world.get::<&mut NeuronEngulfment>(entity) {
            eng.target = Some(target_entity);
        }

        if new_timer > 0.0 {
            continue;
        }

        // Range check: only fire when within fire_range of the target soma.
        let Some(target_pos) = entity_pos(world, target_entity) else { continue };
        if pos.distance(target_pos) > fire_range {
            continue; // timer stays ≤0 and will fire as soon as we enter range
        }

        if let Ok(mut eng) = world.get::<&mut NeuronEngulfment>(entity) {
            eng.shoot_timer = cooldown;
        }
        to_spawn.push((pos, target_entity, damage));
    }

    for (pos, target, damage) in to_spawn {
        world.spawn((
            Position { position: pos },
            AttackProjectile {
                target,
                // Slower than default so the projectile is clearly visible in flight.
                speed: ATTACK_PROJECTILE_SPEED * 0.55,
                health_damage: damage,
                axon_damage: 0.0,
                // Hot magenta — matches macrophage color palette.
                color: glam::Vec3::new(1.0, 0.05, 0.85),
                radius: 1.4,
            },
        ));
    }
}

// ── 4. Burst attacks ──────────────────────────────────────────────────────────

pub fn apply_burst_attacks(world: &mut hecs::World, dt: f32) {
    let attackers: Vec<(hecs::Entity, Vec3, f32, f32, f32, f32, Option<hecs::Entity>)> = world
        .query::<(&Position, &BurstAttack, &MobileUnit)>()
        .iter()
        .map(|(e, (p, b, m))| {
            (e, p.position, b.damage, b.range, b.cooldown, b.cooldown_timer, m.target)
        })
        .collect();

    for (entity, pos, damage, range, cooldown, timer, target) in attackers {
        let new_timer = (timer - dt).max(-1.0);
        if let Ok(mut b) = world.get::<&mut BurstAttack>(entity) {
            b.cooldown_timer = new_timer;
        }

        if new_timer > 0.0 {
            continue;
        }

        let target_entity = match target.filter(|&t| is_alive(world, t)).or_else(|| {
            priority_neuron(world, pos).map(|(e, _)| e)
        }) {
            Some(t) => t,
            None => continue,
        };

        if let Some(target_pos) = entity_pos(world, target_entity) {
            if pos.distance(target_pos) <= range {
                if let Ok(mut ms) = world.get::<&mut MetabolicState>(target_entity) {
                    ms.energy = (ms.energy - damage as f64).max(0.0);
                }
                if let Ok(mut h) = world.get::<&mut Health>(target_entity) {
                    h.current -= damage;
                }
                if let Ok(mut b) = world.get::<&mut BurstAttack>(entity) {
                    b.cooldown_timer = cooldown;
                }
            }
        }
    }
}

// ── 5. Advance attack projectiles ─────────────────────────────────────────────

/// Move all projectiles toward their targets; apply damage and despawn on arrival.
pub fn advance_attack_projectiles(world: &mut hecs::World, dt: f32) {
    let projectiles: Vec<(hecs::Entity, Vec3, hecs::Entity, f32, f32, f32)> = world
        .query::<(&Position, &AttackProjectile)>()
        .iter()
        .map(|(e, (p, a))| {
            (e, p.position, a.target, a.speed, a.health_damage, a.axon_damage)
        })
        .collect();

    let mut moves: Vec<(hecs::Entity, Vec3)> = Vec::new();
    let mut hits: Vec<(hecs::Entity, f32, f32)> = Vec::new(); // (target, health_dmg, axon_dmg)
    let mut to_despawn: Vec<hecs::Entity> = Vec::new();

    for (entity, pos, target, speed, health_dmg, axon_dmg) in projectiles {
        if !is_alive(world, target) {
            to_despawn.push(entity);
            continue;
        }
        let Some(target_pos) = entity_pos(world, target) else {
            to_despawn.push(entity);
            continue;
        };

        let dist = pos.distance(target_pos);
        // Arrive when within 0.8 units of target centre.
        if dist <= 0.8 {
            hits.push((target, health_dmg, axon_dmg));
            to_despawn.push(entity);
        } else {
            let step = speed * dt;
            let dir = (target_pos - pos).normalize_or_zero();
            // Don't overshoot.
            let new_pos = if step >= dist { target_pos } else { pos + dir * step };
            moves.push((entity, new_pos));
        }
    }

    for (entity, new_pos) in moves {
        if let Ok(mut p) = world.get::<&mut Position>(entity) {
            p.position = new_pos;
        }
    }

    for (target, health_dmg, axon_dmg) in hits {
        if health_dmg > 0.0 {
            if let Ok(mut h) = world.get::<&mut Health>(target) {
                h.current -= health_dmg;
            }
        }
        if axon_dmg > 0.0 {
            if let Ok(mut ah) = world.get::<&mut AxonHealth>(target) {
                ah.current -= axon_dmg;
            }
        }
    }

    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}

// ── 6. Glial absorption ───────────────────────────────────────────────────────

pub fn apply_glial_absorption(world: &mut hecs::World, dt: f32) {
    let absorbers: Vec<(Vec3, f32, f32, bool)> = world
        .query::<(&Position, &GlialAbsorption)>()
        .iter()
        .map(|(e, (p, a))| {
            let owned = world
                .get::<&Ownership>(e)
                .map(|o| o.player == PlayerId::Player1)
                .unwrap_or(false);
            (p.position, a.absorb_radius, a.absorb_rate, owned)
        })
        .collect();

    let mobiles: Vec<(hecs::Entity, Vec3, bool)> = world
        .query::<(&Position, &MobileUnit)>()
        .iter()
        .map(|(e, (p, _))| {
            let owned = world
                .get::<&Ownership>(e)
                .map(|o| o.player == PlayerId::Player1)
                .unwrap_or(false);
            (e, p.position, owned)
        })
        .collect();

    for (absorber_pos, radius, rate, absorber_owned) in &absorbers {
        for (mobile_entity, mobile_pos, mobile_owned) in &mobiles {
            if absorber_owned == mobile_owned {
                continue;
            }
            if absorber_pos.distance(*mobile_pos) <= *radius {
                if let Ok(mut health) = world.get::<&mut Health>(*mobile_entity) {
                    health.current -= rate * dt;
                }
            }
        }
    }
}

// ── 7. Despawn dead units ─────────────────────────────────────────────────────

pub fn despawn_dead(world: &mut hecs::World) {
    let dead_units: Vec<hecs::Entity> = world
        .query::<&Health>()
        .iter()
        .filter(|(_, h)| h.is_dead())
        .map(|(e, _)| e)
        .collect();

    let dead_axons: Vec<hecs::Entity> = world
        .query::<&AxonHealth>()
        .iter()
        .filter(|(_, h)| h.is_dead())
        .map(|(e, _)| e)
        .collect();

    for entity in dead_units.into_iter().chain(dead_axons) {
        let _ = world.despawn(entity);
    }
}
