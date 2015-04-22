#include "current.h"

#include "neuronnode.h"

Current::Current(QQuickItem *parent)
    : Entity(parent)
{
    connect(this, &NeuronNode::parentChanged, this, &Current::connectVoltageToParent);
    connectVoltageToParent(parent);
}

Current::~Current()
{
}

double Current::current() const
{
    return m_current;
}

double Current::voltage() const
{
    return m_voltage;
}

void Current::setCurrent(double arg)
{
    if (m_current == arg)
        return;

    m_current = arg;
    emit currentChanged(arg);
}

void Current::setVoltage(double arg)
{
    if (m_voltage == arg)
        return;

    m_voltage = arg;
    emit voltageChanged(arg);
}

void Current::connectVoltageToParent(QQuickItem *parent)
{
    NeuronNode* parentNode = qobject_cast<NeuronNode*>(parent);
    if(!parentNode && parent != 0) {
        qWarning() << "Warning: Parent of Current is not NeuronNode. Will not be able to listen to voltage change events.";
    }
    if(m_previousParent) {
        disconnect(m_previousParent, &NeuronNode::voltageChanged, this, &Current::setVoltage);
    }
    if(parentNode) {
        connect(parentNode, &NeuronNode::voltageChanged, this, &Current::setVoltage);
    }
}

