#include "entity.h"

#include "nodebase.h"

Entity::Entity(QQuickItem *parent)
    : QQuickItem(parent)
{

}

Entity::~Entity()
{

}

bool Entity::hasFired()
{
    return m_hasFired;
}

void Entity::setHasFired(bool fired)
{
    m_hasFired = fired;
}

void Entity::step(double dt)
{
    for(Entity* child : findChildren<Entity*>()) {
        NodeBase* node = qobject_cast<NodeBase*>(child);
        if(node) {
            continue;
        }
        child->step(dt);
    }
    stepEvent(dt);
    emit stepped(dt);
}

void Entity::fire()
{
    m_hasFired = true;
    for(Entity* child : findChildren<Entity*>()) {
        NodeBase* node = qobject_cast<NodeBase*>(child);
        if(node) {
            continue;
        }
        child->fire();
    }
    fireEvent();
    emit fired();
}

void Entity::stimulate(double stimulation)
{
    for(Entity* child : findChildren<Entity*>()) {
        NodeBase* node = qobject_cast<NodeBase*>(child);
        if(node) {
            continue;
        }
        child->stimulate(stimulation);
    }
    stimulateEvent(stimulation);
    emit stimulated(stimulation);
}

void Entity::finalizeStep(double dt)
{
    m_hasFired = false;
    for(Entity* child : findChildren<Entity*>()) {
        child->finalizeStep(dt);
    }
    finalizeStepEvent(dt);
    emit finalizedStep(dt);
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

void Entity::finalizeStepEvent(double dt)
{
    Q_UNUSED(dt);
}

