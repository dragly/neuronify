#include "enginebase.h"

#include "nodebase.h"

EngineBase::EngineBase(QQuickItem *parent)
    : QQuickItem(parent)
{

}

EngineBase::~EngineBase()
{

}

bool EngineBase::hasFired()
{
    return m_hasFired;
}

void EngineBase::setHasFired(bool fired)
{
    m_hasFired = fired;
}

void EngineBase::step(double dt)
{
    for(EngineBase* child : findChildren<EngineBase*>()) {
        NodeBase* node = qobject_cast<NodeBase*>(child);
        if(node) {
            continue;
        }
        child->step(dt);
    }
    stepEvent(dt);
    emit stepped(dt);
}

void EngineBase::fire()
{
    m_hasFired = true;
    for(EngineBase* child : findChildren<EngineBase*>()) {
        NodeBase* node = qobject_cast<NodeBase*>(child);
        if(node) {
            continue;
        }
        child->fire();
    }
    fireEvent();
    emit fired();
}

void EngineBase::stimulate(double stimulation)
{
    for(EngineBase* child : findChildren<EngineBase*>()) {
        NodeBase* node = qobject_cast<NodeBase*>(child);
        if(node) {
            continue;
        }
        child->stimulate(stimulation);
    }
    stimulateEvent(stimulation);
    emit stimulated(stimulation);
}

void EngineBase::finalizeStep(double dt)
{
    m_hasFired = false;
    for(EngineBase* child : findChildren<EngineBase*>()) {
        child->finalizeStep(dt);
    }
    finalizeStepEvent(dt);
    emit finalizedStep(dt);
}

void EngineBase::stepEvent(double dt)
{
    Q_UNUSED(dt);
}

void EngineBase::fireEvent()
{

}

void EngineBase::stimulateEvent(double stimulation)
{
    Q_UNUSED(stimulation);
}

void EngineBase::finalizeStepEvent(double dt)
{
    Q_UNUSED(dt);
}

