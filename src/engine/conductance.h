#ifndef CONDUCTANCE_H
#define CONDUCTANCE_H

#include <QObject>

#include "entity.h"

class Conductance : public Entity
{
    Q_OBJECT
    Q_PROPERTY(double conductance READ conductance WRITE setConductance NOTIFY conductanceChanged)

public:
    Conductance(QQuickItem *parent = 0);
    ~Conductance();

    double conductance() const;

public slots:
    void setConductance(double arg);

signals:
    void conductanceChanged(double arg);

private:
    double m_conductance;

};

#endif // CONDUCTANCE_H
