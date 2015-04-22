#include "conductance.h"

Conductance::Conductance(QQuickItem *parent)
    : Entity(parent)
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
