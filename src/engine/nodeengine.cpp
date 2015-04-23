#include "nodeengine.h"

#include "nodebase.h"

NodeEngine::NodeEngine(QQuickItem *parent)
    : QQuickItem(parent)
{

}

NodeEngine::~NodeEngine()
{

}

double NodeEngine::stimulation() const
{
    return m_stimulation;
}

void NodeEngine::setStimulation(double arg)
{
    if (m_stimulation == arg)
        return;

    m_stimulation = arg;
    emit stimulationChanged(arg);
}

bool NodeEngine::hasFired()
{
    return m_hasFired;
}

void NodeEngine::setHasFired(bool fired)
{
    m_hasFired = fired;
}

void NodeEngine::step(double dt)
{
    for(NodeEngine* child : findChildren<NodeEngine*>()) {
        child->step(dt);
    }
    stepEvent(dt);
    emit stepped(dt);
}

void NodeEngine::fire()
{
    m_hasFired = true;
    for(NodeEngine* child : findChildren<NodeEngine*>()) {
        child->fire();
    }
    fireEvent();
    emit fired();
}

void NodeEngine::stimulate(double stimulation)
{
    for(NodeEngine* child : findChildren<NodeEngine*>()) {
        NodeBase* node = qobject_cast<NodeBase*>(child);
        if(node) {
            continue;
        }
        child->stimulate(stimulation);
    }
    stimulateEvent(stimulation);
    emit stimulated(stimulation);
}

void NodeEngine::finalizeStep(double dt)
{
    m_hasFired = false;
    for(NodeEngine* child : findChildren<NodeEngine*>()) {
        child->finalizeStep(dt);
    }
    finalizeStepEvent(dt);
    emit finalizedStep(dt);
}

void NodeEngine::stepEvent(double dt)
{
    Q_UNUSED(dt);
}

void NodeEngine::fireEvent()
{

}

void NodeEngine::stimulateEvent(double stimulation)
{
    Q_UNUSED(stimulation);
}

void NodeEngine::finalizeStepEvent(double dt)
{
    Q_UNUSED(dt);
}
