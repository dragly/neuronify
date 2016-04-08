#include "adaptationcurrent.h"

#include "neuronengine.h"

/*!
 * \class AdaptationCurrent
 * \inmodule Neuronify
 * \ingroup neuronify-neurons
 * \brief The AdaptationCurrent class produces a current that drives the
 * voltage towards the membrane potential and increases each time the neuron
 * fires.
 */

AdaptationCurrent::AdaptationCurrent(QQuickItem *parent)
    : Current(parent)
{

}

AdaptationCurrent::~AdaptationCurrent()
{

}

double AdaptationCurrent::adaptation() const
{
    return m_adaptation;
}

double AdaptationCurrent::conductance() const
{
    return m_conductance;
}

double AdaptationCurrent::timeConstant() const
{
    return m_timeConstant;
}

void AdaptationCurrent::setAdaptation(double arg)
{
    if (m_adaptation == arg)
        return;

    m_adaptation = arg;
    emit adaptationChanged(arg);
}

void AdaptationCurrent::setConductance(double arg)
{
    if (m_conductance == arg)
        return;

    m_conductance = arg;
    emit conductanceChanged(arg);
}

void AdaptationCurrent::setTimeConstant(double arg)
{
    if (m_timeConstant == arg)
        return;

    m_timeConstant = arg;
    emit timeConstantChanged(arg);
}

void AdaptationCurrent::stepEvent(double dt, bool parentEnabled)
{
    Q_UNUSED(dt);
    if(!parentEnabled) {
        return;
    }
    NeuronEngine* parentNode = qobject_cast<NeuronEngine*>(parent());
    if(!parentNode) {
        qWarning() << "Warning: Parent of Current is not NeuronNode. Cannot find voltage.";
        return;
    }

    double Em = parentNode->restingPotential();
    double V = parentNode->voltage();
    double g = m_conductance;
    double tau = m_timeConstant;

    g = g - g/tau * dt;

    double I = -g * (V - Em);

    setConductance(g);
    setTimeConstant(tau);
    setCurrent(I);
}

void AdaptationCurrent::fireEvent()
{
    m_conductance += m_adaptation;
}

void AdaptationCurrent::resetPropertiesEvent()
{
    m_adaptation = 10.0e-9;
    m_timeConstant = 1000.0e-3;
}

void AdaptationCurrent::resetDynamicsEvent()
{
    m_conductance = 0.0;
}

