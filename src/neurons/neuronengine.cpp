#include "neuronengine.h"

#include <QDebug>
#include <cmath>

#include "current.h"

using namespace std;

/*!
 * \class NeuronEngine
 * \inmodule Neuronify
 * \ingroup neuronify-neurons
 * \brief The NeuronEngine class is a common engine used by most neurons.
 *
 * Neurons have common properties such as the resting membrane potential,
 * possible synaptic input and a voltage.
 * Currents are responsible for changes to the voltage.
 *
 * \l NodeEngine holds a list of all children added in QML.
 * In stepEvent() all the children are iterated, and if they contain a
 * \l Current object, the Current::current() function is called to obtain
 * the current value of the given current.
 * The \l Current class can be subclassed in C++ or QML to define different
 * types of currents, such as \l PassiveCurrent and \l AdaptationCurrent.
 */

NeuronEngine::NeuronEngine(QQuickItem *parent)
    : NodeEngine(parent)
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

double NeuronEngine::threshold() const
{
    return m_threshold;
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

void NeuronEngine::reset()
{
    m_voltage = -100.;
    m_synapticConductance = 0.0;
}

void NeuronEngine::resetVoltage()
{
    m_voltage = m_membraneRestingPotential;
    m_synapticConductance = 0.0;
    emit voltageChanged(m_voltage);
    emit synapticConductanceChanged(m_synapticConductance);
}

void NeuronEngine::initialize()
{
    m_membraneRestingPotential = -65.0;
    m_synapsePotential = 50.0;
    m_clampCurrentEnabled = false;
    m_clampCurrent = 0.0;
}

void NeuronEngine::setThreshold(double threshold)
{
    if (m_threshold == threshold)
        return;

    m_threshold = threshold;
    emit thresholdChanged(threshold);
}

void NeuronEngine::checkFire()
{
    if(m_firedLastTime) {
        setVoltage(m_membraneRestingPotential);
        m_firedLastTime = false;
        return;
    }

    if(m_voltage > m_threshold) {
        fire();
    }
}
