#ifndef CONDUCTANCE_H
#define CONDUCTANCE_H

#include <QObject>

class Conductance : public QObject
{
    Q_OBJECT
    Q_PROPERTY(double conductance READ conductance WRITE setConductance NOTIFY conductanceChanged)

public:
    Conductance(QObject *parent = 0);
    ~Conductance();

    double conductance() const;

public slots:
    void setConductance(double arg);
    void step(double dt);

signals:
    void conductanceChanged(double arg);
    void stepped(double dt);
    void fire();

protected:
    virtual void stepEvent(double dt);

private:
    double m_conductance;

};

#endif // CONDUCTANCE_H
