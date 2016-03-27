#ifndef ADAPTATIONCURRENT_H
#define ADAPTATIONCURRENT_H

#include "current.h"

class AdaptationCurrent : public Current
{
    Q_OBJECT
    Q_PROPERTY(double adaptation READ adaptation WRITE setAdaptation NOTIFY adaptationChanged)
    Q_PROPERTY(double conductance READ conductance NOTIFY conductanceChanged)
    Q_PROPERTY(double timeConstant READ timeConstant WRITE setTimeConstant NOTIFY timeConstantChanged)

public:
    explicit AdaptationCurrent(QQuickItem *parent = 0);
    ~AdaptationCurrent();

    double adaptation() const;
    double conductance() const;

    double timeConstant() const;

signals:
    void adaptationChanged(double arg);
    void conductanceChanged(double arg);

    void timeConstantChanged(double arg);

public slots:
    void setAdaptation(double arg);
    void setConductance(double arg);

    void setTimeConstant(double arg);

protected:
    virtual void stepEvent(double dt) override;
    virtual void fireEvent() override;

private:
    double m_adaptation = 10.0e-6;
    double m_conductance = 0.0;
    double m_timeConstant = 1000.0e-3;
};

#endif // ADAPTATIONCURRENT_H
