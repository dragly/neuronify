#include "adaptationcurrent.h"

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

void AdaptationCurrent::stepEvent(double dt)
{
    Q_UNUSED(dt);
    double Em = m_restingPotential;
    double V = voltage();
    double g = m_conductance;

    g = g - g * dt;

    double I = -g * (V - Em);

    setConductance(g);
    setCurrent(I);
}

void AdaptationCurrent::fireEvent()
{
    m_conductance += m_adaptation;
}

