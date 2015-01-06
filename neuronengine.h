#ifndef NEURONENGINE_H
#define NEURONENGINE_H
#include <QQuickFramebufferObject>

class NeuronEngine : public QObject
{
    Q_OBJECT
    Q_PROPERTY(double voltage READ voltage WRITE setVoltage NOTIFY voltageChanged)
    Q_PROPERTY(double synapticConductance READ synapticConductance WRITE setSynapticConductance NOTIFY synapticConductanceChanged)
    Q_PROPERTY(double adaptationConductance READ adaptationConductance WRITE setAdaptationConductance NOTIFY adaptionConductanceChanged)
    Q_PROPERTY(double membraneRestingPotential READ membraneRestingPotential WRITE setMembraneRestingPotential NOTIFY membraneRestingPotentialChanged)
    Q_PROPERTY(double synapsePotential READ synapsePotential WRITE setSynapsePotential NOTIFY synapsePotentialChanged)
    Q_PROPERTY(bool clampCurrentEnabled READ clampCurrentEnabled WRITE setClampCurrentEnabled NOTIFY clampCurrentEnabledChanged)
    Q_PROPERTY(double clampCurrent READ clampCurrent WRITE setClampCurrent NOTIFY clampCurrentChanged)
    Q_PROPERTY(double cm READ cm WRITE setCm NOTIFY cmChanged)

public:
    NeuronEngine();
    double voltage() const;
    double synapticConductance() const;
    double adaptionConductance() const;
    double membraneRestingPotential() const;
    double synapsePotential() const;
    double clampCurrent() const;
    double cm() const;
    bool clampCurrentEnabled() const;

    double adaptationConductance() const;

public slots:
    void setVoltage(double arg);
    void stepForward(double dt);
    void setSynapticConductance(double arg);
    void setMembraneRestingPotential(double arg);
    void setSynapsePotential(double arg);
    void setClampCurrentEnabled(bool arg);
    void setClampCurrent(double arg);
    void setCm(double arg);
    void setAdaptationConductance(double arg);
    void reset();
    void initialize();

signals:
    void voltageChanged(double arg);
    void synapticConductanceChanged(double arg);
    void membraneRestingPotentialChanged(double arg);
    void synapsePotentialChanged(double arg);
    void clampCurrentEnabledChanged(bool arg);
    void clampCurrentChanged(double arg);
    void cmChanged(double arg);
    void adaptionConductanceChanged(double arg);

private:
    double m_cm;
    double m_voltage;
    double m_membraneRestingPotential;
    double m_synapsePotential;
    double m_synapticConductance;
    double m_adaptationConductance;
    bool m_clampCurrentEnabled;
    double m_clampCurrent;
};

#endif // NEURONENGINE_H
