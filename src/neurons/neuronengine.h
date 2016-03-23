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
    Q_PROPERTY(double restingPotential READ restingPotential WRITE setRestingPotential NOTIFY restingPotentialChanged)
    Q_PROPERTY(double synapsePotential READ synapsePotential WRITE setSynapsePotential NOTIFY synapsePotentialChanged)
    Q_PROPERTY(bool clampCurrentEnabled READ clampCurrentEnabled WRITE setClampCurrentEnabled NOTIFY clampCurrentEnabledChanged)
    Q_PROPERTY(double clampCurrent READ clampCurrent WRITE setClampCurrent NOTIFY clampCurrentChanged)
    Q_PROPERTY(double threshold READ threshold WRITE setThreshold NOTIFY thresholdChanged)

public:
    NeuronEngine(QQuickItem *parent = 0);
    double voltage() const;
    double synapticConductance() const;
    double adaptionConductance() const;
    double restingPotential() const;
    double synapsePotential() const;
    double clampCurrent() const;
    bool clampCurrentEnabled() const;
    double threshold() const;

public slots:
    void setVoltage(double arg);
    void setSynapticConductance(double arg);
    void setRestingPotential(double arg);
    void setSynapsePotential(double arg);
    void setClampCurrentEnabled(bool arg);
    void setClampCurrent(double arg);
    void reset();
    void resetVoltage();
    void initialize();
    void setThreshold(double threshold);

signals:
    void voltageChanged(double arg);
    void synapticConductanceChanged(double arg);
    void restingPotentialChanged(double arg);
    void synapsePotentialChanged(double arg);
    void clampCurrentEnabledChanged(bool arg);
    void clampCurrentChanged(double arg);
    void thresholdChanged(double threshold);

protected:
    virtual void stepEvent(double dt);
    virtual void fireEvent();
    virtual void receiveFireEvent(double fireOutput, NodeEngine *sender);
    virtual void receiveCurrentEvent(double currentOutput, NodeEngine *sender);

private:
    void checkFire();

    double m_voltage = 0.0;
    double m_membraneRestingPotential = 0.0;
    double m_synapsePotential = 0.0;
    double m_synapticConductance = 0.0;
    bool m_clampCurrentEnabled = false;
    double m_clampCurrent = 0.0;
    double m_receivedCurrents = 0.0;
    bool m_firedLastTime = false;
    double m_threshold = 0.0;
};

#endif // NEURONIFY_NEURONENGINE_H
