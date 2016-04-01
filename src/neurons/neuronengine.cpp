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
{
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

double NeuronEngine::synapticPotential() const
{
    return m_synapticPotential;
}

double NeuronEngine::threshold() const
{
    return m_threshold;
}

double NeuronEngine::capacitance() const
{
    return m_capacitance;
}

double NeuronEngine::initialPotential() const
{
    return m_initialPotential;
}

double NeuronEngine::synapticTimeConstant() const
{
    return m_synapticTimeConstant;
}


void NeuronEngine::setVoltage(double arg)
{
    if (m_voltage != arg) {
        m_voltage = arg;
        emit voltageChanged(arg);
    }
}

void NeuronEngine::stepEvent(double dt, bool parentEnabled)
{
    if(!parentEnabled) {
        return;
    }

    checkFire();

    double otherCurrents = 0.0;
    for(Current* current : findChildren<Current*>()) {
        if(current->isEnabled()) {
            otherCurrents += current->current();
        }
    }

    double gs = m_synapticConductance;
    double tau = m_synapticTimeConstant;
    double dgs = -gs / tau * dt;

    double V = m_voltage;
    double Es = m_synapticPotential;
    double synapticCurrents = -gs * (V - Es);

    double totalCurrent = synapticCurrents + otherCurrents + m_receivedCurrents;
    double dV = totalCurrent / m_capacitance * dt;
    m_voltage += dV;

    m_voltage = min(max(m_voltage, -0.2), 0.2);
    m_synapticConductance = gs + dgs;


    emit voltageChanged(m_voltage);
    emit synapticConductanceChanged(m_synapticConductance);

    m_receivedCurrents = 0.0;
}

void NeuronEngine::fireEvent()
{
    setVoltage(m_initialPotential);
    setSynapticConductance(0.0);
}

void NeuronEngine::receiveCurrentEvent(double currentOutput, NodeEngine *sender)
{
    Q_UNUSED(sender);
    if(!isEnabled()) {
        return;
    }
    m_receivedCurrents += currentOutput;
}

void NeuronEngine::receiveFireEvent(double stimulation, NodeEngine *sender)
{
    Q_UNUSED(sender);
    if(!isEnabled()) {
        return;
    }
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

void NeuronEngine::setSynapticPotential(double arg)
{
    if (m_synapticPotential != arg) {
        m_synapticPotential = arg;
        emit synapticPotentialChanged(arg);
    }
}

void NeuronEngine::resetEvent()
{
    setVoltage(m_initialPotential);
    setSynapticConductance(0.0);
}

void NeuronEngine::setThreshold(double threshold)
{
    if (m_threshold == threshold)
        return;

    m_threshold = threshold;
    emit thresholdChanged(threshold);
}

void NeuronEngine::setCapacitance(double capacitance)
{
    if (m_capacitance == capacitance)
        return;

    m_capacitance = capacitance;
    emit capacitanceChanged(capacitance);
}

void NeuronEngine::setInitialPotential(double postFirePotential)
{
    if (m_initialPotential == postFirePotential)
        return;

    m_initialPotential = postFirePotential;
    emit initialPotentialChanged(postFirePotential);
}

void NeuronEngine::setSynapticTimeConstant(double synapticTimeConstant)
{
    if (m_synapticTimeConstant == synapticTimeConstant)
        return;

    m_synapticTimeConstant = synapticTimeConstant;
    emit synapticTimeConstantChanged(synapticTimeConstant);
}

void NeuronEngine::checkFire()
{
    if(m_voltage > m_threshold) {
        fire();
    }
}
