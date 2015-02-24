#include "neuronengine.h"
#include <QDebug>
NeuronEngine::NeuronEngine() :
    m_cm(0),
    m_voltage(0),
    m_membraneRestingPotential(0),
    m_synapsePotential(0),
    m_synapticConductance(0),
    m_adaptationConductance(0),
    m_clampCurrentEnabled(false),
    m_clampCurrent(0)
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


double NeuronEngine::membraneRestingPotential() const
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

double NeuronEngine::adaptationConductance() const
{
    return m_adaptationConductance;
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

void NeuronEngine::step(double dt)
{

    double gs = m_synapticConductance;
    double dgs = -gs / 1.0 * dt;
    double gadapt = m_adaptationConductance;
    double dgadapt = -gadapt / 1.0 * dt;

    double V = m_voltage;
    double Rm = 1.0;
    double Em = m_membraneRestingPotential;
    double Es = m_synapsePotential;
    double Is = gs * (V - Es);
    double Iadapt = gadapt * (V - Em);
    double Iauto = 0.0;
    if(m_clampCurrentEnabled) {
        Iauto = m_clampCurrent;
    }

    double voltageChange = 1.0 / m_cm * (- (V - Em) / Rm) - Is - Iadapt + Iauto;
    double dV = voltageChange * dt;
    m_voltage += dV;

    m_synapticConductance = gs + dgs;
//        m_synapticConductance = Math.max(-0.5, m_synapticConductance);
    m_adaptationConductance = gadapt + dgadapt;

    emit voltageChanged(m_voltage);
    emit synapticConductanceChanged(m_synapticConductance);
    emit adaptionConductanceChanged(m_adaptationConductance);
}

void NeuronEngine::setSynapticConductance(double arg)
{
    if (m_synapticConductance != arg) {
        m_synapticConductance = arg;
        emit synapticConductanceChanged(arg);
    }
}


void NeuronEngine::setMembraneRestingPotential(double arg)
{
    if (m_membraneRestingPotential != arg) {
        m_membraneRestingPotential = arg;
        emit membraneRestingPotentialChanged(arg);
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

void NeuronEngine::setAdaptationConductance(double arg)
{
    if (m_adaptationConductance != arg) {
        m_adaptationConductance = arg;
        emit adaptionConductanceChanged(arg);
    }
}

void NeuronEngine::reset()
{
    m_voltage = -100.;
    m_synapticConductance = 0.0;
    m_adaptationConductance = 0.0;
}

void NeuronEngine::initialize()
{
    m_cm = 1.0;
    m_membraneRestingPotential = -65.0;
    m_synapsePotential = 50.0;
    m_clampCurrentEnabled = false;
    m_clampCurrent = 0.0;
}

