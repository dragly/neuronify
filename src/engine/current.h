#ifndef CURRENT_H
#define CURRENT_H

#include <QQuickItem>

#include "entity.h"

class NeuronNode;
class Current : public Entity
{
    Q_OBJECT
    Q_PROPERTY(double current READ current WRITE setCurrent NOTIFY currentChanged)
    Q_PROPERTY(double voltage READ voltage NOTIFY voltageChanged)

public:
    explicit Current(QQuickItem* parent = 0);
    ~Current();

    double current() const;
    double voltage() const;

signals:
    void currentChanged(double arg);
    void voltageChanged(double arg);

public slots:
    void setCurrent(double arg);

private slots:
    void setVoltage(double arg);
    void connectVoltageToParent(QQuickItem* parent);

private:
    double m_current = 0.0;
    double m_voltage = 0.0;

    NeuronNode* m_previousParent = nullptr;
};

#endif // CURRENT_H
