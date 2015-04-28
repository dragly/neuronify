#include "neuronengine.h"

#include <QDebug>
#include <cmath>

#include "current.h"

using namespace std;

NeuronEngine::NeuronEngine(QQuickItem *parent)
    : NodeEngine(parent)
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

double NeuronEngine::voltage() const
{
    return m_voltage;
}

double NeuronEngine::synapticConductance() const
{
    return m_synapticConductance;
}

double NeuronEngine::restingPotential() const
{
    return m_membraneRestingPotential;
}

double NeuronEngine::synapsePotential() const
{
    return m_synapsePotential;
}

bool NeuronEngine::clampCurrentEnabled() const
{
    return m_clampCurrentEnabled;
}

double NeuronEngine::cm() const
{
    return m_cm;
}

double NeuronEngine::clampCurrent() const
{
    return m_clampCurrent;
}

void NeuronEngine::setVoltage(double arg)
{
    if (m_voltage != arg) {
        m_voltage = arg;
        emit voltageChanged(arg);
    }
}

void NeuronEngine::stepEvent(double dt)
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

    double voltageChange = synapticCurrents + otherCurrents + m_receivedCurrents;
    double dV = voltageChange * dt;
    m_voltage += dV;

    m_voltage = min(max(m_voltage, -200.0), 200.0);

    m_synapticConductance = gs + dgs;

    emit voltageChanged(m_voltage);
    emit synapticConductanceChanged(m_synapticConductance);

    m_receivedCurrents = 0.0;
}

void NeuronEngine::fireEvent()
{
    setVoltage(100.0);
    setSynapticConductance(0.0);
    m_firedLastTime = true;
}

void NeuronEngine::receiveCurrentEvent(double currentOutput)
{
    m_receivedCurrents += currentOutput;
}

void NeuronEngine::receiveFireEvent(double stimulation)
{
    m_synapticConductance += stimulation;
}

void NeuronEngine::setSynapticConductance(double arg)
{
    if (m_synapticConductance != arg) {
        m_synapticConductance = arg;
        emit synapticConductanceChanged(arg);
    }
}

void NeuronEngine::setRestingPotential(double arg)
{
    if (m_membraneRestingPotential != arg) {
        m_membraneRestingPotential = arg;
        emit restingPotentialChanged(arg);
    }
}

void NeuronEngine::setSynapsePotential(double arg)
{
    if (m_synapsePotential != arg) {
        m_synapsePotential = arg;
        emit synapsePotentialChanged(arg);
    }
}

void NeuronEngine::setClampCurrentEnabled(bool arg)
{
    if (m_clampCurrentEnabled != arg) {
        m_clampCurrentEnabled = arg;
        emit clampCurrentEnabledChanged(arg);
    }
}

void NeuronEngine::setClampCurrent(double arg)
{
    if (m_clampCurrent != arg) {
        m_clampCurrent = arg;
        emit clampCurrentChanged(arg);
    }
}

void NeuronEngine::setCm(double arg)
{
    if (m_cm != arg) {
        m_cm = arg;
        emit cmChanged(arg);
    }
}

void NeuronEngine::reset()
{
    m_voltage = -100.;
    m_synapticConductance = 0.0;
}

void NeuronEngine::initialize()
{
    m_cm = 1.0;
    m_membraneRestingPotential = -65.0;
    m_synapsePotential = 50.0;
    m_clampCurrentEnabled = false;
    m_clampCurrent = 0.0;
}

void NeuronEngine::checkFire()
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
