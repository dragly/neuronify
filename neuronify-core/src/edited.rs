use hecs::{Component, Entity, EntityBuilder, EntityRef, World};

pub struct Edited {
    pub copy: fn(EntityRef, &mut EntityBuilder),
    pub swap: fn(&World, Entity, &mut EntityBuilder),
    pub restore: fn(&mut EntityBuilder, &mut EntityBuilder),
}

pub const fn edited<T: Component + Clone>() -> Edited {
    Edited {
        copy: copy::<T>,
        swap: swap::<T>,
        restore: restore::<T>,
    }
}

pub trait Settings<C: Component>: Component {
    fn of(component: &C) -> Self;
    fn swap(&mut self, component: &mut C);
}

pub const fn edited_settings<C: Component, S: Settings<C>>() -> Edited {
    Edited {
        copy: copy_settings::<C, S>,
        swap: swap_settings::<C, S>,
        restore: restore_settings::<C, S>,
    }
}

fn restore_settings<C: Component, S: Settings<C>>(
    whole: &mut EntityBuilder,
    edited: &mut EntityBuilder,
) {
    if let (Some(recorded), Some(component)) =
        (edited.get_mut::<&mut S>(), whole.get_mut::<&mut C>())
    {
        recorded.swap(component);
    }
}

fn copy_settings<C: Component, S: Settings<C>>(entity: EntityRef, edited: &mut EntityBuilder) {
    if let Some(component) = entity.get::<&C>() {
        edited.add(S::of(&component));
    }
}

fn swap_settings<C: Component, S: Settings<C>>(
    world: &World,
    entity: Entity,
    edited: &mut EntityBuilder,
) {
    if let (Some(recorded), Ok(mut current)) =
        (edited.get_mut::<&mut S>(), world.get::<&mut C>(entity))
    {
        recorded.swap(&mut current);
    }
}

fn copy<T: Component + Clone>(entity: EntityRef, edited: &mut EntityBuilder) {
    if let Some(component) = entity.get::<&T>() {
        edited.add((*component).clone());
    }
}

fn restore<T: Component + Clone>(whole: &mut EntityBuilder, edited: &mut EntityBuilder) {
    if let Some(recorded) = edited.get::<&T>() {
        whole.add(recorded.clone());
    }
}

fn swap<T: Component>(world: &World, entity: Entity, edited: &mut EntityBuilder) {
    if let (Some(recorded), Ok(mut current)) =
        (edited.get_mut::<&mut T>(), world.get::<&mut T>(entity))
    {
        std::mem::swap(recorded, &mut *current);
    }
}
