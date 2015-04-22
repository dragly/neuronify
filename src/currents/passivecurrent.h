#ifndef PASSIVECURRENT_H
#define PASSIVECURRENT_H

#include "../engine/current.h"

class PassiveCurrent : public Current
{
    Q_OBJECT
    Q_PROPERTY(double resistance READ resistance WRITE setResistance NOTIFY resistanceChanged)
    Q_PROPERTY(double capacitance READ capacitance WRITE setCapacitance NOTIFY capacitanceChanged)
    Q_PROPERTY(double restingPotential READ restingPotential WRITE setRestingPotential NOTIFY restingPotentialChanged)

public:
    explicit PassiveCurrent(QQuickItem *parent = 0);
    ~PassiveCurrent();

    double resistance() const;
    double capacitance() const;
    double restingPotential() const;

signals:
    void resistanceChanged(double arg);
    void capacitanceChanged(double arg);
    void restingPotentialChanged(double arg);

public slots:
    void setResistance(double arg);
    void setCapacitance(double arg);
    void setRestingPotential(double arg);

protected:
    virtual void stepEvent(double dt);

private:
    double m_resistance = 1.0;
    double m_capacitance = 1.0;
    double m_restingPotential = -65.0;
};

#endif // PASSIVECURRENT_H
