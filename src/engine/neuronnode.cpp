#include "neuronnode.h"
#include <QDebug>

#include "current.h"

NeuronNode::NeuronNode(QQuickItem *parent)
    : Node(parent)
    , m_cm(0)
    , m_voltage(0)
    , m_membraneRestingPotential(0)
    , m_synapsePotential(0)
    , m_synapticConductance(0)
    , m_clampCurrentEnabled(false)
    , m_clampCurrent(0)
{
    initialize();
    reset();
}

double NeuronNode::voltage() const
{
    return m_voltage;
}

double NeuronNode::synapticConductance() const
{
    return m_synapticConductance;
}

double NeuronNode::restingPotential() const
{
    return m_membraneRestingPotential;
}

double NeuronNode::synapsePotential() const
{
    return m_synapsePotential;
}

bool NeuronNode::clampCurrentEnabled() const
{
    return m_clampCurrentEnabled;
}

double NeuronNode::cm() const
{
    return m_cm;
}

double NeuronNode::clampCurrent() const
{
    return m_clampCurrent;
}

void NeuronNode::setVoltage(double arg)
{
    if (m_voltage != arg) {
        m_voltage = arg;
        emit voltageChanged(arg);
    }
}

void NeuronNode::stepEvent(double dt)
{
    checkFire();

    double otherCurrents = 0.0;
    for(Current* current : findChildren<Current*>()) {
        if(current->isEnabled()) {
            otherCurrents += current->current();
        }
    }

    double gs = m_synapticConductance;
    double dgs = -gs / 1.0 * dt;

    double V = m_voltage;
    double Es = m_synapsePotential;
    double synapticCurrents = -gs * (V - Es);

    double voltageChange = synapticCurrents + otherCurrents;
    double dV = voltageChange * dt;
    m_voltage += dV;

    m_synapticConductance = gs + dgs;

    emit voltageChanged(m_voltage);
    emit synapticConductanceChanged(m_synapticConductance);
}

void NeuronNode::fireEvent()
{
    setVoltage(voltage() + 100.0);
    m_firedLastTime = true;
}

void NeuronNode::stimulateEvent(double stimulation)
{
    m_synapticConductance += stimulation;
}

void NeuronNode::setSynapticConductance(double arg)
{
    if (m_synapticConductance != arg) {
        m_synapticConductance = arg;
        emit synapticConductanceChanged(arg);
    }
}

void NeuronNode::setRestingPotential(double arg)
{
    if (m_membraneRestingPotential != arg) {
        m_membraneRestingPotential = arg;
        emit restingPotentialChanged(arg);
    }
}

void NeuronNode::setSynapsePotential(double arg)
{
    if (m_synapsePotential != arg) {
        m_synapsePotential = arg;
        emit synapsePotentialChanged(arg);
    }
}

void NeuronNode::setClampCurrentEnabled(bool arg)
{
    if (m_clampCurrentEnabled != arg) {
        m_clampCurrentEnabled = arg;
        emit clampCurrentEnabledChanged(arg);
    }
}

void NeuronNode::setClampCurrent(double arg)
{
    if (m_clampCurrent != arg) {
        m_clampCurrent = arg;
        emit clampCurrentChanged(arg);
    }
}

void NeuronNode::setCm(double arg)
{
    if (m_cm != arg) {
        m_cm = arg;
        emit cmChanged(arg);
    }
}

void NeuronNode::reset()
{
    m_voltage = -100.;
    m_synapticConductance = 0.0;
}

void NeuronNode::initialize()
{
    m_cm = 1.0;
    m_membraneRestingPotential = -65.0;
    m_synapsePotential = 50.0;
    m_clampCurrentEnabled = false;
    m_clampCurrent = 0.0;
}

void NeuronNode::checkFire()
{
    if(m_firedLastTime) {
        setVoltage(m_membraneRestingPotential);
        m_firedLastTime = false;
        return;
    }

    if(m_voltage > 0.0) {
        fire();
    }
}
