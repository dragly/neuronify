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

void PassiveCurrent::setResistance(double arg)
{
    if (m_resistance == arg)
        return;

    m_resistance = arg;
    emit resistanceChanged(arg);
}

void PassiveCurrent::stepEvent(double dt, bool parentEnabled)
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
    double I = -1.0 / m_resistance * (V - Em);
    setCurrent(I);
}
