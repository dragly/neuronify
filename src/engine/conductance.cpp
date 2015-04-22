#include "conductance.h"

Conductance::Conductance(QObject *parent)
    : QObject(parent)
{

}

Conductance::~Conductance()
{

}

double Conductance::conductance() const
{
    return m_conductance;
}

void Conductance::setConductance(double arg)
{
    if (m_conductance == arg)
        return;

    m_conductance = arg;
    emit conductanceChanged(arg);
}

void Conductance::step(double dt)
{
    stepEvent(dt);
    emit stepped(dt);
}

void Conductance::stepEvent(double dt) {
    Q_UNUSED(dt);
}
