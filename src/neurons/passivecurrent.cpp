#include "passivecurrent.h"

#include "neuronengine.h"

/*!
 * \class PassiveCurrent
 * \inmodule Neuronify
 * \ingroup neuronify-neurons
 * \brief The PassiveCurrent class defines a current that drives the
 * \l NeuronEngine towards the defined resting membrane potential.
 */

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

    recalculateOneOverResistanceTimesCapacitance();
    emit resistanceChanged(arg);
}

void PassiveCurrent::setCapacitance(double arg)
{
    if (m_capacitance == arg)
        return;

    m_capacitance = arg;

    recalculateOneOverResistanceTimesCapacitance();
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

    double oneOverRmCm = m_oneOverResistanceTimesCapacitance;
    double Em = parentNode->restingPotential();
    double V = parentNode->voltage();
    double I = - oneOverRmCm * (V - Em);
    setCurrent(I);
}

void PassiveCurrent::recalculateOneOverResistanceTimesCapacitance()
{
    m_oneOverResistanceTimesCapacitance = 1.0 / (m_resistance * m_capacitance);
}
