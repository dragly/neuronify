#include "adaptationcurrent.h"

#include "neuronengine.h"

/*!
 * \class AdaptationCurrent
 * \inmodule Neuronify
 * \ingroup neuronify-neurons
 * \brief Adaptive Current calculates the neuron conductance based on an
 * adaptive rule.
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

double AdaptationCurrent::restingPotential() const
{
    return m_restingPotential;
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

void AdaptationCurrent::setRestingPotential(double arg)
{
    if (m_restingPotential == arg)
        return;

    m_restingPotential = arg;
    emit restingPotentialChanged(arg);
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

void AdaptationCurrent::stepEvent(double dt)
{
    Q_UNUSED(dt);

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

