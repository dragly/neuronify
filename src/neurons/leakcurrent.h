#ifndef PASSIVECURRENT_H
#define PASSIVECURRENT_H

#include "current.h"

class LeakCurrent : public Current
{
    Q_OBJECT
    Q_PROPERTY(double resistance READ resistance WRITE setResistance NOTIFY resistanceChanged)

public:
    explicit LeakCurrent(QQuickItem *parent = 0);
    ~LeakCurrent();

    double resistance() const;

signals:
    void resistanceChanged(double arg);

public slots:
    void setResistance(double arg);

protected:
    virtual void stepEvent(double dt, bool parentEnabled) override;
    virtual void resetPropertiesEvent() override;

private:
    double m_resistance = 100.0e6;
};

#endif // PASSIVECURRENT_H
