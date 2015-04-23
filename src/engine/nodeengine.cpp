#include "nodeengine.h"

#include "nodebase.h"

NodeEngine::NodeEngine(QQuickItem *parent)
    : QQuickItem(parent)
{

}

NodeEngine::~NodeEngine()
{

}

double NodeEngine::fireOutput() const
{
    return m_stimulation;
}

void NodeEngine::setFireOutput(double arg)
{
    if (m_stimulation == arg)
        return;

    m_stimulation = arg;
    emit fireOutputChanged(arg);
}

void NodeEngine::setCurrentOutput(double arg)
{
    if (m_currentStimulation == arg)
        return;

    m_currentStimulation = arg;
    emit currentOutputChanged(arg);
}

bool NodeEngine::hasFired()
{
    return m_hasFired;
}

void NodeEngine::setHasFired(bool fired)
{
    m_hasFired = fired;
}

double NodeEngine::currentOutput() const
{
    return m_currentStimulation;
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

void NodeEngine::receiveFire(double stimulation)
{
    for(NodeEngine* child : findChildren<NodeEngine*>()) {
        child->receiveFire(stimulation);
    }
    receiveFireEvent(stimulation);
    emit receivedFire(stimulation);
}

void NodeEngine::receiveCurrent(double current)
{
    for(NodeEngine* child : findChildren<NodeEngine*>()) {
        child->receiveCurrent(current);
    }
    receiveCurrentEvent(current);
    emit receivedFire(current);
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

void NodeEngine::receiveFireEvent(double fireOutput)
{
    Q_UNUSED(fireOutput);
}

void NodeEngine::receiveCurrentEvent(double currentOutput)
{
    Q_UNUSED(currentOutput);
}

void NodeEngine::finalizeStepEvent(double dt)
{
    Q_UNUSED(dt);
}
