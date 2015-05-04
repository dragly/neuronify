#ifndef ADAPTATIONCURRENT_H
#define ADAPTATIONCURRENT_H

#include "current.h"

class AdaptationCurrent : public Current
{
    Q_OBJECT
    Q_PROPERTY(double adaptation READ adaptation WRITE setAdaptation NOTIFY adaptationChanged)
    Q_PROPERTY(double restingPotential READ restingPotential WRITE setRestingPotential NOTIFY restingPotentialChanged)
    Q_PROPERTY(double conductance READ conductance NOTIFY conductanceChanged)

public:
    explicit AdaptationCurrent(QQuickItem *parent = 0);
    ~AdaptationCurrent();

    double adaptation() const;
    double restingPotential() const;
    double conductance() const;

signals:
    void adaptationChanged(double arg);
    void restingPotentialChanged(double arg);
    void conductanceChanged(double arg);

public slots:
    void setAdaptation(double arg);
    void setRestingPotential(double arg);
    void setConductance(double arg);

protected:
    virtual void stepEvent(double dt) override;
    virtual void fireEvent() override;

private:
    double m_adaptation = 1.0;
    double m_restingPotential = -65.0;
    double m_conductance = 0.0;
};

#endif // ADAPTATIONCURRENT_H
