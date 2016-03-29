#ifndef RATEENGINE_H
#define RATEENGINE_H

#include "../core/nodeengine.h"

#include <QQuickItem>
#include <QQmlListProperty>
#include <vector>
#include "../utility/mathhelper.h"
using namespace std;

class RateEngine : public NodeEngine
{
    Q_OBJECT
    Q_PROPERTY(double firingRate READ firingRate WRITE setFiringRate NOTIFY firingRateChanged)
    Q_PROPERTY(double windowDuration READ windowDuration WRITE setWindowDuration NOTIFY windowDurationChanged)

    Q_PROPERTY(int neuronCount READ neuronCount WRITE setNeuronCount NOTIFY neuronCountChanged)

public:
    RateEngine(QQuickItem *parent = 0);

    double firingRate() const;
    double windowDuration() const;

    int neuronCount() const;

public slots:
    void setFiringRate(double firingRate);
    void setWindowDuration(double windowDuration);
    void setNeuronCount(int neuronCount);

signals:
    void firingRateChanged(double firingRate);
    void windowDurationChanged(double windowDuration);
    void neuronCountChanged(int neuronCount);

protected:
    virtual void receiveFireEvent(double fireOutput, NodeEngine *sender);

private:
    void computeFiringRate();

    int m_neuronCount = 0;
    double m_firingRate = 0.0;

    vector<double> m_spikeTimes;
    double m_time = 0.0;
    double m_windowDuration = 100e-3;


protected:
    virtual void stepEvent(double dt) override;
};

#endif // RATEENGINE_H
