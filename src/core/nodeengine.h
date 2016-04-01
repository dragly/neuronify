#ifndef NODEENGINE_H
#define NODEENGINE_H

#include "neuronifyobject.h"

#include <QQuickItem>

class NodeEngine : public NeuronifyObject
{
    Q_OBJECT
    Q_PROPERTY(double fireOutput READ fireOutput WRITE setFireOutput NOTIFY fireOutputChanged)
    Q_PROPERTY(double currentOutput READ currentOutput WRITE setCurrentOutput NOTIFY currentOutputChanged)

public:
    explicit NodeEngine(QQuickItem *parent = 0);
    ~NodeEngine();

    double fireOutput() const;
    double currentOutput() const;

    bool hasFired();
    void setHasFired(bool arg);
    Q_INVOKABLE void reset();

signals:
    void stepped(double dt, bool parentEnabled);
    void fired();
    void receivedFire(double stimulation, NodeEngine *sender);
    void receivedCurrent(double current, NodeEngine *sender);
    void finalizedStep(double dt);
    void fireOutputChanged(double arg);
    void currentOutputChanged(double arg);
    void resetted();

public slots:
    void step(double dt, bool parentEnabled);
    void fire();
    void receiveFire(double fireOutput, NodeEngine *sender);
    void receiveCurrent(double currentOutput, NodeEngine *sender);
    void finalizeStep(double dt);
    void setFireOutput(double arg);
    void setCurrentOutput(double arg);

protected:
    virtual void stepEvent(double dt, bool parentEnabled);
    virtual void fireEvent();
    virtual void receiveFireEvent(double fireOutput, NodeEngine *sender);
    virtual void receiveCurrentEvent(double currentOutput, NodeEngine *sender);
    virtual void finalizeStepEvent(double dt);
    virtual void resetEvent();

private:
    bool m_hasFired = false;
    double m_fireOutput = 1.0;
    double m_currentOutput = 0.0;
};

#endif // NODEENGINE_H
