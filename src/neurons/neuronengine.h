#ifndef NEURONIFY_NEURONENGINE_H
#define NEURONIFY_NEURONENGINE_H

#include <QQuickItem>
#include <QQmlListProperty>

#include "../core/nodeengine.h"

class NeuronEngine : public NodeEngine
{
    Q_OBJECT

    Q_PROPERTY(double voltage READ voltage WRITE setVoltage NOTIFY voltageChanged)
    Q_PROPERTY(double synapticConductance READ synapticConductance WRITE setSynapticConductance NOTIFY synapticConductanceChanged)
    Q_PROPERTY(double synapticTimeConstant READ synapticTimeConstant WRITE setSynapticTimeConstant NOTIFY synapticTimeConstantChanged)
    Q_PROPERTY(double restingPotential READ restingPotential WRITE setRestingPotential NOTIFY restingPotentialChanged)
    Q_PROPERTY(double initialPotential READ initialPotential WRITE setInitialPotential NOTIFY initialPotentialChanged)
    Q_PROPERTY(double synapticPotential READ synapticPotential WRITE setSynapticPotential NOTIFY synapticPotentialChanged)
    Q_PROPERTY(double threshold READ threshold WRITE setThreshold NOTIFY thresholdChanged)
    Q_PROPERTY(double capacitance READ capacitance WRITE setCapacitance NOTIFY capacitanceChanged)
    Q_PROPERTY(double refractoryPeriod READ refractoryPeriod WRITE setRefractoryPeriod NOTIFY refractoryPeriodChanged)

public:
    NeuronEngine(QQuickItem *parent = 0);
    double voltage() const;
    double synapticConductance() const;
    double adaptionConductance() const;
    double restingPotential() const;
    double synapticPotential() const;
    double threshold() const;
    double capacitance() const;
    double initialPotential() const;
    double synapticTimeConstant() const;

    double refractoryPeriod() const;

public slots:
    void setVoltage(double arg);
    void setSynapticConductance(double arg);
    void setRestingPotential(double arg);
    void setSynapticPotential(double arg);
    void resetEvent();
    void setThreshold(double threshold);
    void setCapacitance(double capacitance);
    void setInitialPotential(double initialPotential);
    void setSynapticTimeConstant(double synapticTimeConstant);

    void setRefractoryPeriod(double refractoryPeriod);

signals:
    void voltageChanged(double arg);
    void synapticConductanceChanged(double arg);
    void restingPotentialChanged(double arg);
    void synapticPotentialChanged(double arg);
    void thresholdChanged(double threshold);
    void capacitanceChanged(double capacitance);
    void initialPotentialChanged(double initialPotential);
    void synapticTimeConstantChanged(double synapticTimeConstant);

    void refractoryPeriodChanged(double refractoryPeriod);

protected:
    virtual void stepEvent(double dt);
    virtual void fireEvent();
    virtual void receiveFireEvent(double fireOutput, NodeEngine *sender);
    virtual void receiveCurrentEvent(double currentOutput, NodeEngine *sender);

private:
    void checkFire();

    double m_voltage = -65.0e-3;
    double m_membraneRestingPotential = -65.0e-3;
    double m_synapticPotential = 50.0e-3;
    double m_synapticConductance = 0.0;
    double m_receivedCurrents = 0.0;
    double m_threshold = 0.0e-3;
    double m_capacitance = 1.0e-6;
    double m_initialPotential = -80.0e-3;
    double m_synapticTimeConstant = 10.0e-3;

    double m_refractoryPeriod = 20e-3;
    double m_timeSinceLastFiring = 0.0;
};

#endif // NEURONIFY_NEURONENGINE_H
