#ifndef PASSIVECURRENT_H
#define PASSIVECURRENT_H

#include "current.h"

class PassiveCurrent : public Current
{
    Q_OBJECT
    Q_PROPERTY(double resistance READ resistance WRITE setResistance NOTIFY resistanceChanged)

public:
    explicit PassiveCurrent(QQuickItem *parent = 0);
    ~PassiveCurrent();

    double resistance() const;

signals:
    void resistanceChanged(double arg);

public slots:
    void setResistance(double arg);

protected:
    virtual void stepEvent(double dt, bool parentEnabled);

private:
    double m_resistance = 10.0e3;
};

#endif // PASSIVECURRENT_H
