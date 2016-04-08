#include "nodeengine.h"

#include "nodebase.h"

/*!
 * \class NodeEngine
 * \inmodule Neuronify
 * \ingroup neuronify-core
 * \brief The NodeEngine class is used to perform operations in for \l NodeBase.
 *
 * All NodeBase objects can hold a pointer to a NodeEngine.
 * The NodeEngine performs the logic for the given node and should
 * hold all information about the state of the node.
 *
 * \sa Node, NodeBase
 */

NodeEngine::NodeEngine(QQuickItem *parent)
    : NeuronifyObject(parent)
{
}

NodeEngine::~NodeEngine()
{
}

void NodeEngine::setCurrentOutput(double arg)
{
    if (m_currentOutput == arg)
        return;

    m_currentOutput = arg;
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
    return m_currentOutput;
}

void NodeEngine::step(double dt, bool parentEnabled)
{
    bool enable = isEnabled() && parentEnabled;
    for(NodeEngine* child : findChildren<NodeEngine*>()) {
        child->step(dt, enable);
    }
    stepEvent(dt, enable);
    emit stepped(dt, enable);
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

void NodeEngine::receiveFire(NodeEngine *sender)
{
    for(NodeEngine* child : findChildren<NodeEngine*>()) {
        child->receiveFire(sender);
    }
    receiveFireEvent(sender);
    emit receivedFire(sender);
}

void NodeEngine::receiveCurrent(double current, NodeEngine *sender)
{
    for(NodeEngine* child : findChildren<NodeEngine*>()) {
        child->receiveCurrent(current, sender);
    }
    receiveCurrentEvent(current, sender);
    emit receivedCurrent(current, sender);
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

void NodeEngine::stepEvent(double dt, bool parentEnabled)
{
    Q_UNUSED(dt);
}

void NodeEngine::fireEvent()
{
}

void NodeEngine::receiveFireEvent(NodeEngine *sender)
{
    Q_UNUSED(sender);
}

void NodeEngine::receiveCurrentEvent(double currentOutput, NodeEngine *sender)
{
    Q_UNUSED(currentOutput);
    Q_UNUSED(sender);
}

void NodeEngine::finalizeStepEvent(double dt)
{
    Q_UNUSED(dt);
}

void NodeEngine::resetDynamicsEvent()
{
    m_hasFired = false;
}
