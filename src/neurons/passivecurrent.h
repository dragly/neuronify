#ifndef PASSIVECURRENT_H
#define PASSIVECURRENT_H

#include "current.h"

class PassiveCurrent : public Current
{
    Q_OBJECT
    Q_PROPERTY(double resistance READ resistance WRITE setResistance NOTIFY resistanceChanged)
    Q_PROPERTY(double capacitance READ capacitance WRITE setCapacitance NOTIFY capacitanceChanged)

public:
    explicit PassiveCurrent(QQuickItem *parent = 0);
    ~PassiveCurrent();

    double resistance() const;
    double capacitance() const;

signals:
    void resistanceChanged(double arg);
    void capacitanceChanged(double arg);

public slots:
    void setResistance(double arg);
    void setCapacitance(double arg);

protected:
    virtual void stepEvent(double dt);

private:
    double m_resistance = 1.0;
    double m_capacitance = 1.0;
};

#endif // PASSIVECURRENT_H
