#ifndef CURRENT_H
#define CURRENT_H

#include <QQuickItem>

#include "enginebase.h"

class NeuronEngine;
class Current : public EngineBase
{
    Q_OBJECT
    Q_PROPERTY(double current READ current WRITE setCurrent NOTIFY currentChanged)

public:
    explicit Current(QQuickItem* parent = 0);
    ~Current();

    double current() const;

signals:
    void currentChanged(double arg);

public slots:
    void setCurrent(double arg);

private:
    double m_current = 0.0;
};

#endif // CURRENT_H
