//! Combat systems — one function per unit mechanic.
//!
//! Units fire visible projectiles toward their targets; damage is applied on
//! projectile arrival. Call order each frame:
//! move → fire (axon/engulf/burst) → advance_projectiles → absorption → despawn.

use glam::Vec3;
use neuronify_core::{Compartment, LeakyDynamics, LeakyNeuron, OriginNeuron, GeneratorDynamics, Position, VisualRadius};

use crate::components::{
    AttackProjectile, AxonCutter, AxonHealth, BurstAttack, Dying, Faction, GlialAbsorption,
    Health, MacrophageUnit, MetabolicState, MicroglialCell, MobileUnit, NeuronEngulfment,
    EnemySpawnPoint, NeuronSpawnType, NeuronSpawner, Ownership, PlayerId, ReactiveAstrocyte, SlowEffect,
};
use crate::spawning;
use crate::constants::{
    ASTROCYTE_STAGGER_COOLDOWN, ASTROCYTE_STAGGER_SLOW_DURATION, ASTROCYTE_STAGGER_SLOW_FACTOR,
    ATTACK_PROJECTILE_SPEED, COMBAT_PRIORITY_RANGE, DEATH_DURATION,
    REACTIVE_ASTROCYTE_ABSORB_RADIUS,
};

// ── Target finders ────────────────────────────────────────────────────────────

fn entity_pos(world: &hecs::World, entity: hecs::Entity) -> Option<Vec3> {
    world.get::<&Position>(entity).ok().map(|p| p.position)
}

/// Returns true only for entities that exist AND are not in a dying animation.
fn is_alive(world: &hecs::World, entity: hecs::Entity) -> bool {
    world.contains(entity) && world.get::<&Dying>(entity).is_err()
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

fn player_astrocyte_positions(world: &hecs::World) -> Vec<(hecs::Entity, Vec3)> {
    world
        .query::<(&Position, &ReactiveAstrocyte, &Ownership)>()
        .iter()
        .filter(|(_, (_, _, o))| o.player == PlayerId::Player1)
        .map(|(e, (p, _, _))| (e, p.position))
        .collect()
}

// ── 1. Move mobile units ──────────────────────────────────────────────────────

pub fn move_mobile_units(world: &mut hecs::World, dt: f32) {
    let axon_targets = player_axon_positions(world);
    let neuron_targets = player_neuron_positions(world);
    let astrocyte_targets = player_astrocyte_positions(world);

    let mut moves: Vec<(hecs::Entity, Vec3)> = Vec::new();
    let mut target_updates: Vec<(hecs::Entity, Option<hecs::Entity>)> = Vec::new();

    for (entity, (pos, mobile)) in world.query::<(&Position, &MobileUnit)>().iter() {
        let from = pos.position;
        let faction = mobile.faction;

        // Player manual override takes highest priority.
        let valid_manual = mobile
            .manual_target
            .filter(|&t| is_alive(world, t))
            .and_then(|t| entity_pos(world, t).map(|p| (t, p)));

        let valid_target = mobile
            .target
            .filter(|&t| is_alive(world, t))
            .and_then(|t| entity_pos(world, t).map(|p| (t, p)));

        let target = if let Some(t) = valid_manual {
            Some(t)
        } else if let Some(t) = valid_target {
            Some(t)
        } else {
            let has_axon_cutter = world.get::<&AxonCutter>(entity).is_ok();
            let has_engulfment = world.get::<&NeuronEngulfment>(entity).is_ok();
            let has_burst = world.get::<&BurstAttack>(entity).is_ok();

            if faction != Faction::Biological {
                // Priority 1: If being drained by an astrocyte, attack it.
                let inside_astrocyte = astrocyte_targets.iter()
                    .find(|(ae, ap)| {
                        let r = world.get::<&GlialAbsorption>(*ae)
                            .map(|g| g.absorb_radius)
                            .unwrap_or(0.0);
                        from.distance(*ap) <= r
                    })
                    .copied();
                if let Some((ae, ap)) = inside_astrocyte {
                    Some((ae, ap))
                } else {
                    // Priority 2: Nearby player combat units (within COMBAT_PRIORITY_RANGE).
                    let nearby_bio = enemy_mobile_positions(world, faction)
                        .into_iter()
                        .filter(|(_, p)| from.distance(*p) <= COMBAT_PRIORITY_RANGE)
                        .collect::<Vec<_>>();
                    if !nearby_bio.is_empty() {
                        nearest_of(&nearby_bio, from)
                    } else if has_axon_cutter {
                        nearest_of(&axon_targets, from)
                    } else if has_engulfment {
                        nearest_of(&neuron_targets, from)
                    } else if has_burst {
                        priority_neuron(world, from)
                    } else {
                        nearest_of(&neuron_targets, from)
                    }
                }
            } else {
                let enemies = enemy_mobile_positions(world, Faction::Biological);
                nearest_of(&enemies, from)
            }
        };

        if let Some((target_entity, target_pos)) = target {
            // When targeting an astrocyte: stop at absorb_radius − 1.5 so the
            // macrophage is still inside the absorb field (getting drained) but
            // the bodies don't visually overlap (combined radii ≈ 5.0 units,
            // gap = ABSORB_RADIUS − 1.5 − 5.0 ≈ 1.5 units clear).
            // For all other targets: halt just inside fire_range as before.
            let is_astrocyte_target = world.get::<&ReactiveAstrocyte>(target_entity).is_ok();
            let standoff = if is_astrocyte_target {
                (REACTIVE_ASTROCYTE_ABSORB_RADIUS - 1.5).max(0.0)
            } else {
                world
                    .get::<&NeuronEngulfment>(entity)
                    .map(|e| (e.fire_range - 0.5).max(0.0))
                    .unwrap_or(0.0)
            };
            let dist = from.distance(target_pos);
            if dist > standoff {
                let dir = (target_pos - from).normalize_or_zero();
                let speed_scale = world
                    .get::<&SlowEffect>(entity)
                    .map(|s| s.factor)
                    .unwrap_or(1.0);
                let step = mobile.speed * speed_scale * dt;
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

// ── 1b. Unit-to-unit repulsion ────────────────────────────────────────────────

/// Position-correction repulsion so mobile units push each other apart.
/// Called after move_mobile_units; does not use SpatialDynamics.
pub fn apply_unit_repulsion(world: &mut hecs::World) {
    let units: Vec<(hecs::Entity, Vec3, f32)> = world
        .query::<(&Position, &VisualRadius, &MobileUnit)>()
        .iter()
        .map(|(e, (p, vr, _))| (e, p.position, vr.radius))
        .collect();

    let mut pushes: Vec<(hecs::Entity, Vec3)> = Vec::new();
    for i in 0..units.len() {
        for j in (i + 1)..units.len() {
            let (ea, pa, ra) = units[i];
            let (eb, pb, rb) = units[j];
            let min_dist = ra + rb;
            let delta = pa - pb;
            let dist = delta.length();
            if dist < min_dist && dist > 0.001 {
                // Full gap closure each call; app.rs calls this twice per frame to converge.
                let push = delta.normalize_or_zero() * (min_dist - dist);
                pushes.push((ea,  push * 0.5));
                pushes.push((eb, -push * 0.5));
            }
        }
    }
    for (e, push) in pushes {
        if let Ok(mut p) = world.get::<&mut Position>(e) {
            p.position += push;
        }
    }
}

// ── 1c. Neuron-driven unit spawning ──────────────────────────────────────────

/// Each frame: neurons with a NeuronSpawner component that fired recently
/// (time_since_fire < combat_dt × 1.5) and whose cooldown has elapsed will
/// spawn a combat unit at spawn_offset from the soma.
pub fn apply_neuron_spawning(world: &mut hecs::World, dt: f32) {
    let fire_threshold = dt as f64 * 1.5;

    let mut fired_this_frame: Vec<hecs::Entity> = Vec::new();
    for (e, d) in world.query::<&LeakyDynamics>().iter() {
        if d.time_since_fire < fire_threshold {
            fired_this_frame.push(e);
        }
    }
    for (e, d) in world.query::<&GeneratorDynamics>().iter() {
        if d.time_since_fire < fire_threshold {
            fired_this_frame.push(e);
        }
    }

    // Tick all cooldown timers regardless of firing.
    for (_, spawner) in world.query_mut::<&mut NeuronSpawner>() {
        spawner.timer = (spawner.timer - dt).max(0.0);
    }

    let mut to_spawn: Vec<(Vec3, Faction, NeuronSpawnType)> = Vec::new();
    for entity in fired_this_frame {
        if let Ok(mut spawner) = world.get::<&mut NeuronSpawner>(entity) {
            if spawner.timer <= 0.0 {
                if let Ok(pos) = world.get::<&Position>(entity) {
                    let spawn_pos = pos.position + spawner.spawn_offset;
                    spawner.timer = spawner.cooldown;
                    to_spawn.push((spawn_pos, spawner.faction, spawner.spawn_type.clone()));
                }
            }
        }
    }

    for (pos, faction, spawn_type) in to_spawn {
        match spawn_type {
            NeuronSpawnType::MicroglialCell => {
                spawning::spawn_microglial_cell(world, pos, faction);
            }
            NeuronSpawnType::TCell => {
                spawning::spawn_tCell(world, pos, faction);
            }
            NeuronSpawnType::Macrophage => {
                spawning::spawn_macrophage(world, pos, faction);
            }
        }
    }
}

// ── 1b. Timer-based enemy spawn points ───────────────────────────────────────

/// Ticks `EnemySpawnPoint` timers and spawns units when ready.
/// Works without the neural simulation (no LIF step required).
pub fn apply_enemy_spawn_points(world: &mut hecs::World, dt: f32) {
    let mut to_spawn: Vec<(glam::Vec3, Faction, NeuronSpawnType)> = Vec::new();

    for (_, (point, pos)) in world.query_mut::<(&mut EnemySpawnPoint, &Position)>() {
        point.timer = (point.timer - dt).max(0.0);
        if point.timer <= 0.0 {
            to_spawn.push((pos.position + point.spawn_offset, point.faction, point.spawn_type.clone()));
            point.timer = point.cooldown;
        }
    }

    for (pos, faction, spawn_type) in to_spawn {
        match spawn_type {
            NeuronSpawnType::MicroglialCell => spawning::spawn_microglial_cell(world, pos, faction),
            NeuronSpawnType::TCell => spawning::spawn_tCell(world, pos, faction),
            NeuronSpawnType::Macrophage => spawning::spawn_macrophage(world, pos, faction),
        };
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

    let mut to_spawn: Vec<(Vec3, hecs::Entity, f32, f32, f32)> = Vec::new(); // pos, target, health_dmg, axon_dmg, speed

    for (entity, pos, damage, cooldown, timer, mobile_target) in cutters {
        let new_timer = timer - dt;
        if let Ok(mut c) = world.get::<&mut AxonCutter>(entity) {
            c.shoot_timer = new_timer;
        }

        if new_timer > 0.0 {
            continue;
        }

        // Target: use movement target if valid (axon OR mobile unit), else nearest axon.
        let target = mobile_target
            .filter(|&t| is_alive(world, t))
            .filter(|&t| {
                world.get::<&AxonHealth>(t).is_ok() || world.get::<&Health>(t).is_ok()
            })
            .or_else(|| nearest_of(&axon_targets, pos).map(|(e, _)| e));

        let Some(target_entity) = target else { continue };

        if let Ok(mut c) = world.get::<&mut AxonCutter>(entity) {
            c.shoot_timer = cooldown;
        }
        // Route damage to health or axon depending on what the target has.
        let (health_dmg, axon_dmg) = if world.get::<&AxonHealth>(target_entity).is_ok() {
            (0.0, damage)
        } else {
            (damage, 0.0)
        };
        to_spawn.push((pos, target_entity, health_dmg, axon_dmg, ATTACK_PROJECTILE_SPEED * 1.2));
    }

    for (pos, target, health_dmg, axon_dmg, speed) in to_spawn {
        world.spawn((
            Position { position: pos },
            AttackProjectile {
                target,
                speed,
                health_damage: health_dmg,
                axon_damage: axon_dmg,
                slow_duration: 0.0,
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

        // Retaliation: if this macrophage is inside an astrocyte absorb zone, shoot the astrocyte.
        let astro_targets = player_astrocyte_positions(world);
        let threatening = astro_targets.iter()
            .find(|(ae, ap)| {
                let r = world.get::<&GlialAbsorption>(*ae)
                    .map(|g| g.absorb_radius)
                    .unwrap_or(0.0);
                pos.distance(*ap) <= r
            })
            .map(|(ae, _)| *ae);

        // Find/validate target: retaliation > stored > nearest neuron.
        let target = threatening
            .or_else(|| stored_target.filter(|&t| is_alive(world, t)))
            .or_else(|| nearest_of(&player_neuron_positions(world), pos).map(|(e, _)| e));
        let Some(target_entity) = target else { continue };

        if let Ok(mut eng) = world.get::<&mut NeuronEngulfment>(entity) {
            eng.target = Some(target_entity);
        }

        if new_timer > 0.0 {
            continue;
        }

        // Range check: fire at neurons only within fire_range;
        // astrocytes (already inside absorb zone) use the full absorb radius as effective range.
        let Some(target_pos) = entity_pos(world, target_entity) else { continue };
        let effective_range = if world.get::<&ReactiveAstrocyte>(target_entity).is_ok() {
            REACTIVE_ASTROCYTE_ABSORB_RADIUS
        } else {
            fire_range
        };
        if pos.distance(target_pos) > effective_range {
            continue;
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
                slow_duration: 0.0,
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
    let projectiles: Vec<(hecs::Entity, Vec3, hecs::Entity, f32, f32, f32, f32)> = world
        .query::<(&Position, &AttackProjectile)>()
        .iter()
        .map(|(e, (p, a))| {
            (e, p.position, a.target, a.speed, a.health_damage, a.axon_damage, a.slow_duration)
        })
        .collect();

    let mut moves: Vec<(hecs::Entity, Vec3)> = Vec::new();
    let mut hits: Vec<(hecs::Entity, f32, f32, f32)> = Vec::new(); // (target, health_dmg, axon_dmg, slow_duration)
    let mut to_despawn: Vec<hecs::Entity> = Vec::new();

    for (entity, pos, target, speed, health_dmg, axon_dmg, slow_dur) in projectiles {
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
            hits.push((target, health_dmg, axon_dmg, slow_dur));
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

    for (target, health_dmg, axon_dmg, slow_dur) in hits {
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
        if slow_dur > 0.0 {
            // Apply or refresh SlowEffect on target.
            let has_slow = world.get::<&SlowEffect>(target).is_ok();
            if has_slow {
                if let Ok(mut se) = world.get::<&mut SlowEffect>(target) {
                    se.timer = se.timer.max(slow_dur);
                }
            } else {
                let _ = world.insert_one(target, SlowEffect { timer: slow_dur, factor: ASTROCYTE_STAGGER_SLOW_FACTOR });
            }
        }
    }

    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}

// ── 6. Glial absorption ───────────────────────────────────────────────────────

pub fn apply_glial_absorption(world: &mut hecs::World, dt: f32) {
    // Snapshot absorbers: (entity, pos, radius, rate, owned, stagger_timer)
    let absorbers: Vec<(hecs::Entity, Vec3, f32, f32, bool, f32)> = world
        .query::<(&Position, &GlialAbsorption)>()
        .iter()
        .map(|(e, (p, a))| {
            let owned = world
                .get::<&Ownership>(e)
                .map(|o| o.player == PlayerId::Player1)
                .unwrap_or(false);
            (e, p.position, a.absorb_radius, a.absorb_rate, owned, a.stagger_timer)
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

    // (astrocyte_pos, target_entity) — stagger bolts to spawn this frame
    let mut stagger_bolts: Vec<(Vec3, hecs::Entity)> = Vec::new();

    for (absorber_entity, absorber_pos, radius, rate, absorber_owned, timer) in &absorbers {
        let new_timer = (timer - dt).max(-1.0);
        if let Ok(mut ga) = world.get::<&mut GlialAbsorption>(*absorber_entity) {
            ga.stagger_timer = new_timer;
        }

        for (mobile_entity, mobile_pos, mobile_owned) in &mobiles {
            if absorber_owned == mobile_owned {
                continue;
            }
            if absorber_pos.distance(*mobile_pos) <= *radius {
                // Continuous passive drain (non-stagger component).
                if let Ok(mut health) = world.get::<&mut Health>(*mobile_entity) {
                    health.current -= rate * dt;
                }
                // When the stagger cooldown expires, queue a bolt toward this enemy.
                if new_timer <= 0.0 {
                    stagger_bolts.push((*absorber_pos, *mobile_entity));
                }
            }
        }

        // Reset timer once per absorber (after we've queued all bolts for it).
        if new_timer <= 0.0 && stagger_bolts.iter().any(|(ap, _)| *ap == *absorber_pos) {
            if let Ok(mut ga) = world.get::<&mut GlialAbsorption>(*absorber_entity) {
                ga.stagger_timer = ASTROCYTE_STAGGER_COOLDOWN;
            }
        }
    }

    for (from_pos, target) in stagger_bolts {
        world.spawn((
            Position { position: from_pos },
            AttackProjectile {
                target,
                speed: ATTACK_PROJECTILE_SPEED * 0.9,
                health_damage: 0.0,
                axon_damage: 0.0,
                slow_duration: ASTROCYTE_STAGGER_SLOW_DURATION,
                // Warm gold — visually distinct from cyan (microglia) and magenta (macrophage).
                color: glam::Vec3::new(1.0, 0.82, 0.1),
                radius: 0.6,
            },
        ));
    }
}

// ── 7. Tick slow effects ──────────────────────────────────────────────────────

/// Decrement SlowEffect timers; remove the component when it expires.
pub fn tick_slow_effects(world: &mut hecs::World, dt: f32) {
    let expired: Vec<hecs::Entity> = world
        .query::<&SlowEffect>()
        .iter()
        .filter_map(|(e, se)| {
            if se.timer <= dt {
                Some(e)
            } else {
                None
            }
        })
        .collect();

    let ticking: Vec<hecs::Entity> = world
        .query::<&SlowEffect>()
        .iter()
        .filter_map(|(e, se)| if se.timer > dt { Some(e) } else { None })
        .collect();

    for e in ticking {
        if let Ok(mut se) = world.get::<&mut SlowEffect>(e) {
            se.timer -= dt;
        }
    }
    for e in expired {
        let _ = world.remove_one::<SlowEffect>(e);
    }
}

// ── 8. Death transitions and ticking ─────────────────────────────────────────

/// Transition dead combat units into the dying-animation state; immediately
/// despawn everything else (axons, non-unit entities).
pub fn despawn_dead(world: &mut hecs::World) {
    // Collect combat units whose health just hit zero and haven't started dying yet.
    let newly_dead: Vec<hecs::Entity> = world
        .query::<&Health>()
        .iter()
        .filter(|(e, h)| {
            h.is_dead()
                && world.get::<&Dying>(*e).is_err()
                && (world.get::<&MicroglialCell>(*e).is_ok()
                    || world.get::<&MacrophageUnit>(*e).is_ok()
                    || world.get::<&ReactiveAstrocyte>(*e).is_ok())
        })
        .map(|(e, _)| e)
        .collect();

    for entity in newly_dead {
        // Remove health so is_alive() returns false and we can't target this unit.
        world.remove_one::<Health>(entity).ok();
        // Also stop it from moving or acting.
        world.remove_one::<MobileUnit>(entity).ok();
        // Seed for scatter variety: entity ID gives stable per-unit variety.
        let seed = (entity.id() as f32) * 2.399_f32;
        let _ = world.insert_one(entity, Dying {
            timer: 0.0,
            duration: DEATH_DURATION,
            seed,
        });
    }

    // Immediately despawn dead non-unit entities (axon segments, neurons, etc.).
    let dead_health: Vec<hecs::Entity> = world
        .query::<&Health>()
        .iter()
        .filter(|(e, h)| {
            h.is_dead()
                && world.get::<&Dying>(*e).is_err()
        })
        .map(|(e, _)| e)
        .collect();

    let dead_axons: Vec<hecs::Entity> = world
        .query::<&AxonHealth>()
        .iter()
        .filter(|(_, h)| h.is_dead())
        .map(|(e, _)| e)
        .collect();

    for entity in dead_health.into_iter().chain(dead_axons) {
        let _ = world.despawn(entity);
    }
}

/// Advance all dying-animation timers; despawn units whose animation has finished.
pub fn tick_dying_units(world: &mut hecs::World, dt: f32) {
    let mut to_despawn = Vec::new();
    for (entity, dying) in world.query::<&mut Dying>().iter() {
        dying.timer += dt;
        if dying.timer >= dying.duration {
            to_despawn.push(entity);
        }
    }
    for entity in to_despawn {
        let _ = world.despawn(entity);
    }
}
