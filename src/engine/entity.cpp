#include "entity.h"

Entity::Entity(QQuickItem *parent)
    : QQuickItem(parent)
{

}

Entity::~Entity()
{

}

void Entity::step(double dt)
{
    for(Entity* child : findChildren<Entity*>()) {
        child->step(dt);
    }
    stepEvent(dt);
    emit stepped(dt);
}

void Entity::fire()
{
    for(Entity* child : findChildren<Entity*>()) {
        child->fire();
    }
    fireEvent();
    emit fired();
}

void Entity::stimulate(double stimulation)
{
    for(Entity* child : findChildren<Entity*>()) {
        child->stimulate(stimulation);
    }
    stimulateEvent(stimulation);
    emit stimulated(stimulation);
}

void Entity::stepEvent(double dt)
{
    Q_UNUSED(dt);
}

void Entity::fireEvent()
{

}

void Entity::stimulateEvent(double stimulation)
{
    Q_UNUSED(stimulation);
}

