#include "leakcurrent.h"

#include "neuronengine.h"

/*!
 * \class LeakCurrent
 * \inmodule Neuronify
 * \ingroup neuronify-neurons
 * \brief The LeakCurrent class defines a current that drives the
 * \l NeuronEngine towards the defined resting membrane potential.
 */

LeakCurrent::LeakCurrent(QQuickItem *parent)
    : Current(parent)
{

}

LeakCurrent::~LeakCurrent()
{

}

double LeakCurrent::resistance() const
{
    return m_resistance;
}

void LeakCurrent::setResistance(double arg)
{
    if (m_resistance == arg)
        return;

    m_resistance = arg;
    emit resistanceChanged(arg);
}

void LeakCurrent::stepEvent(double dt, bool parentEnabled)
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

void LeakCurrent::resetPropertiesEvent()
{
    setResistance(100.0e6);
}
