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

double NeuronEngine::synapsePotential() const
{
    return m_synapsePotential;
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

double NeuronEngine::firingRate() const
{
    return m_firingRate;
}

double NeuronEngine::binLength() const
{
    return m_binLength;
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
    m_window +=dt;

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
    double Es = m_synapsePotential;
    double synapticCurrents = -gs * (V - Es);

    double totalCurrent = synapticCurrents + otherCurrents + m_receivedCurrents;
    double dV = totalCurrent / m_capacitance * dt;
    m_voltage += dV;

    m_voltage = min(max(m_voltage, -0.2), 0.2);
    m_synapticConductance = gs + dgs;

    qDebug() << m_binLength;
    if(m_window > m_binLength * 0.4e-3){
        m_firingRate = m_spikeCount / m_window;
        m_spikeCount = 0;
        m_window = 0.0;
        emit firingRateChanged(m_firingRate);
    }

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
    m_receivedCurrents += currentOutput;
}

void NeuronEngine::receiveFireEvent(double stimulation, NodeEngine *sender)
{
    Q_UNUSED(sender);
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

void NeuronEngine::setFiringRate(double firingRate)


{
    if (m_firingRate == firingRate)
        return;

    m_firingRate = firingRate;
    emit firingRateChanged(firingRate);
}

void NeuronEngine::setBinLength(double binLength)
{
    if (m_binLength == binLength)
        return;

    m_binLength = binLength;
    emit binLengthChanged(binLength);
}

void NeuronEngine::checkFire()
{
    if(m_voltage > m_threshold) {
        fire();
        m_spikeCount +=1;
    }

}
