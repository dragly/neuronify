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
    Q_PROPERTY(double synapsePotential READ synapsePotential WRITE setSynapsePotential NOTIFY synapsePotentialChanged)
    Q_PROPERTY(double threshold READ threshold WRITE setThreshold NOTIFY thresholdChanged)
    Q_PROPERTY(double capacitance READ capacitance WRITE setCapacitance NOTIFY capacitanceChanged)

public:
    NeuronEngine(QQuickItem *parent = 0);
    double voltage() const;
    double synapticConductance() const;
    double adaptionConductance() const;
    double restingPotential() const;
    double synapsePotential() const;
    double threshold() const;
    double capacitance() const;
    double initialPotential() const;

    double synapticTimeConstant() const;

public slots:
    void setVoltage(double arg);
    void setSynapticConductance(double arg);
    void setRestingPotential(double arg);
    void setSynapsePotential(double arg);
    void reset();
    void resetVoltage();
    void setThreshold(double threshold);
    void setCapacitance(double capacitance);
    void setInitialPotential(double initialPotential);

    void setSynapticTimeConstant(double synapticTimeConstant);

signals:
    void voltageChanged(double arg);
    void synapticConductanceChanged(double arg);
    void restingPotentialChanged(double arg);
    void synapsePotentialChanged(double arg);
    void thresholdChanged(double threshold);
    void capacitanceChanged(double capacitance);
    void initialPotentialChanged(double initialPotential);

    void synapticTimeConstantChanged(double synapticTimeConstant);

protected:
    virtual void stepEvent(double dt);
    virtual void fireEvent();
    virtual void receiveFireEvent(double fireOutput, NodeEngine *sender);
    virtual void receiveCurrentEvent(double currentOutput, NodeEngine *sender);

private:
    void checkFire();

    double m_voltage = -65.0e-3;
    double m_membraneRestingPotential = -65.0e-3;
    double m_synapsePotential = 50.0e-3;
    double m_synapticConductance = 0.0;
    double m_receivedCurrents = 0.0;
    double m_threshold = 0.0e-3;
    double m_capacitance = 1.0e-6;
    double m_initialPotential = -80.0e-3;
    double m_synapticTimeConstant = 10.0e-3;
};

#endif // NEURONIFY_NEURONENGINE_H
