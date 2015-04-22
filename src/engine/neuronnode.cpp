#include "neuronnode.h"
#include <QDebug>

#include "conductance.h"
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

double NeuronNode::membraneRestingPotential() const
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
    double conductances = 0.0;
    for(Conductance* conductance : findChildren<Conductance*>()) {
        if(conductance->isEnabled()) {
            conductances += conductance->conductance();
        }
    }
    double otherCurrents = 0.0;
    for(Current* current : findChildren<Current*>()) {
        if(current->isEnabled()) {
            otherCurrents += current->current();
        }
    }

    double gs = m_synapticConductance;
    double dgs = -gs / 1.0 * dt;

    double V = m_voltage;
    double Rm = 1.0;
    double Em = m_membraneRestingPotential;
    double Es = m_synapsePotential;
    double synapticCurrents = -gs * (V - Es);
    double conductanceCurrents = conductances * (V - Em);

    double voltageChange = synapticCurrents + conductanceCurrents + otherCurrents;
    double dV = voltageChange * dt;
    m_voltage += dV;

    m_synapticConductance = gs + dgs;

    emit voltageChanged(m_voltage);
    emit synapticConductanceChanged(m_synapticConductance);
}

void NeuronNode::setSynapticConductance(double arg)
{
    if (m_synapticConductance != arg) {
        m_synapticConductance = arg;
        emit synapticConductanceChanged(arg);
    }
}

void NeuronNode::setMembraneRestingPotential(double arg)
{
    if (m_membraneRestingPotential != arg) {
        m_membraneRestingPotential = arg;
        emit membraneRestingPotentialChanged(arg);
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

