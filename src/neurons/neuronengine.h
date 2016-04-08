#ifndef NEURONIFY_NEURONENGINE_H
#define NEURONIFY_NEURONENGINE_H

#include <QQuickItem>
#include <QQmlListProperty>

#include "../core/nodeengine.h"

class NeuronEngine : public NodeEngine
{
    Q_OBJECT

    Q_PROPERTY(double voltage READ voltage WRITE setVoltage NOTIFY voltageChanged)
    Q_PROPERTY(double restingPotential READ restingPotential WRITE setRestingPotential NOTIFY restingPotentialChanged)
    Q_PROPERTY(double initialPotential READ initialPotential WRITE setInitialPotential NOTIFY initialPotentialChanged)
    Q_PROPERTY(double threshold READ threshold WRITE setThreshold NOTIFY thresholdChanged)
    Q_PROPERTY(double capacitance READ capacitance WRITE setCapacitance NOTIFY capacitanceChanged)

public:
    NeuronEngine(QQuickItem *parent = 0);
    double voltage() const;
    double adaptionConductance() const;
    double restingPotential() const;
    double threshold() const;
    double capacitance() const;
    double initialPotential() const;

public slots:
    void setVoltage(double arg);
    void setRestingPotential(double arg);
    void setThreshold(double threshold);
    void setCapacitance(double capacitance);
    void setInitialPotential(double initialPotential);

signals:
    void voltageChanged(double arg);
    void restingPotentialChanged(double arg);
    void thresholdChanged(double threshold);
    void capacitanceChanged(double capacitance);
    void initialPotentialChanged(double initialPotential);


protected:
    virtual void stepEvent(double dt, bool parentEnabled);
    virtual void fireEvent();
    virtual void receiveCurrentEvent(double currentOutput, NodeEngine *sender);
    virtual void resetPropertiesEvent() override;
    virtual void resetDynamicsEvent() override;

private:
    void checkFire();

    double m_voltage = 0.0;
    double m_restingPotential = 0.0;
    double m_initialPotential = 0.0;
    double m_threshold = 0.0;
    double m_capacitance = 0.0;
    double m_receivedCurrents = 0.0;


};

#endif // NEURONIFY_NEURONENGINE_H
