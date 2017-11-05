#ifndef COMPARTMENTENGINE_H
#define COMPARTMENTENGINE_H

#include "../core/nodeengine.h"


class CompartmentEngine : public NodeEngine
{
    Q_OBJECT

    Q_PROPERTY(double voltage READ voltage WRITE setVoltage NOTIFY voltageChanged)
    Q_PROPERTY(double capacitance READ capacitance WRITE setCapacitance NOTIFY capacitanceChanged)

public:
    CompartmentEngine();

    double voltage() const;

    double capacitance() const;

signals:

    void voltageChanged(double voltage);

    void capacitanceChanged(double capacitance);

public slots:
    void setVoltage(double voltage);
    void setCapacitance(double capacitance);

    // NeuronifyObject interface
protected:
    virtual void resetDynamicsEvent() override;
    virtual void resetPropertiesEvent() override;

    // NodeEngine interface
protected:
    virtual void stepEvent(double dt, bool parentEnabled) override;
    virtual void receiveCurrentEvent(double currentOutput, NodeEngine *sender) override;

private:
    double m_voltage = 0.0;
    double m_capacitance = 0.0;
    double m_receivedCurrents = 0.0;
};

#endif // COMPARTMENTENGINE_H
