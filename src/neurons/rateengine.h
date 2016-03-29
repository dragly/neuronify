#ifndef RATEENGINE_H
#define RATEENGINE_H

#include "../core/nodeengine.h"

#include <QQuickItem>
#include <QQmlListProperty>

class RateEngine : public NodeEngine
{
    Q_OBJECT
    Q_PROPERTY(double firingRate READ firingRate WRITE setFiringRate NOTIFY firingRateChanged)
    Q_PROPERTY(double binLength READ binLength WRITE setBinLength NOTIFY binLengthChanged)

    Q_PROPERTY(int neuronCount READ neuronCount WRITE setNeuronCount NOTIFY neuronCountChanged)

public:
    RateEngine(QQuickItem *parent = 0);

    double firingRate() const;
    double binLength() const;

    int neuronCount() const;

public slots:
    void setFiringRate(double firingRate);
    void setBinLength(double binLength);
    void setNeuronCount(int neuronCount);

signals:
    void firingRateChanged(double firingRate);
    void binLengthChanged(double binLength);
    void neuronCountChanged(int neuronCount);

protected:
    virtual void receiveFireEvent(double fireOutput, NodeEngine *sender);

private:
    double m_spikeCount = 0.0;
    double m_window = 0.0;
    double m_firingRate = 0.0;
    double m_binLength = 100e-3;

    // NodeEngine interface
    int m_neuronCount = 0;

protected:
    virtual void stepEvent(double dt) override;
};

#endif // RATEENGINE_H
