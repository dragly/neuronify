#include "passivecurrent.h"

PassiveCurrent::PassiveCurrent(QQuickItem *parent)
    : Current(parent)
{

}

PassiveCurrent::~PassiveCurrent()
{

}

double PassiveCurrent::resistance() const
{
    return m_resistance;
}

double PassiveCurrent::capacitance() const
{
    return m_capacitance;
}

double PassiveCurrent::restingPotential() const
{
    return m_restingPotential;
}

void PassiveCurrent::setResistance(double arg)
{
    if (m_resistance == arg)
        return;

    m_resistance = arg;
    emit resistanceChanged(arg);
}

void PassiveCurrent::setCapacitance(double arg)
{
    if (m_capacitance == arg)
        return;

    m_capacitance = arg;
    emit capacitanceChanged(arg);
}

void PassiveCurrent::setRestingPotential(double arg)
{
    if (m_restingPotential == arg)
        return;

    m_restingPotential = arg;
    emit restingPotentialChanged(arg);
}

void PassiveCurrent::stepEvent(double dt)
{
    Q_UNUSED(dt);
    double Rm = m_resistance;
    double Cm = m_capacitance;
    double Em = m_restingPotential;
    double V = voltage();
    double newCurrent = -1.0 / (Rm * Cm) * (V - Em);
    setCurrent(newCurrent);
}

