#include "compartmentengine.h"

#include "../currents/current.h"

CompartmentEngine::CompartmentEngine()
{

}

double CompartmentEngine::voltage() const
{
    return m_voltage;
}

double CompartmentEngine::capacitance() const
{
    return m_capacitance;
}

void CompartmentEngine::setVoltage(double voltage)
{
    if (qFuzzyCompare(m_voltage, voltage))
        return;

    m_voltage = voltage;
    emit voltageChanged(m_voltage);
}

void CompartmentEngine::setCapacitance(double capacitance)
{
    if (qFuzzyCompare(m_capacitance, capacitance))
        return;

    m_capacitance = capacitance;
    emit capacitanceChanged(m_capacitance);
}


void CompartmentEngine::resetDynamicsEvent()
{
    setVoltage(-65e-3);
    m_receivedCurrents = 0.0;
}

void CompartmentEngine::resetPropertiesEvent()
{
    setCapacitance(1e-6);
}

void CompartmentEngine::stepEvent(double dt, bool parentEnabled)
{
    if(!parentEnabled) {
        return;
    }

    double otherCurrents = 0.0;

    // TODO do this only when children have changed
    for(Current* current : findChildren<Current*>()) {
        if(current->isEnabled()) {
            otherCurrents += current->current();
        }
    }

    dt = 0.01; // TODO fix

    double totalCurrent = otherCurrents + m_receivedCurrents;
    double dV = totalCurrent / m_capacitance * dt;
    m_voltage += dV;

    emit voltageChanged(m_voltage);

    m_receivedCurrents = 0.0;
}

void CompartmentEngine::receiveCurrentEvent(double currentOutput, NodeEngine *sender)
{
    Q_UNUSED(sender);
    if(!isEnabled()) {
        return;
    }
    m_receivedCurrents += currentOutput;
}
