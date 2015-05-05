#include "passivecurrent.h"

#include "neuronengine.h"

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

void PassiveCurrent::stepEvent(double dt)
{
    Q_UNUSED(dt);

    NeuronEngine* parentNode = qobject_cast<NeuronEngine*>(parent());
    if(!parentNode) {
        qWarning() << "Warning: Parent of Current is not NeuronNode. Cannot find voltage.";
        return;
    }

    double Rm = m_resistance;
    double Cm = m_capacitance;
    double Em = parentNode->restingPotential();
    double V = parentNode->voltage();
    double I = -1.0 / (Rm * Cm) * (V - Em);
    setCurrent(I);
}
