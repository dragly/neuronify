#ifndef NODEENGINE_H
#define NODEENGINE_H

#include "neuronifyobject.h"
#include "../io/propertygroup.h"

#include <QQuickItem>

class NodeEngine : public NeuronifyObject
{
    Q_OBJECT
    Q_PROPERTY(double currentOutput READ currentOutput WRITE setCurrentOutput NOTIFY currentOutputChanged)

public:
    explicit NodeEngine(QQuickItem *parent = 0);
    ~NodeEngine();

    double currentOutput() const;

    bool hasFired();
    void setHasFired(bool arg);
    Q_INVOKABLE void reset();

signals:
    void stepped(double dt, bool parentEnabled);
    void fired();
    void receivedFire(NodeEngine *sender);
    void receivedCurrent(double current, NodeEngine *sender);
    void finalizedStep(double dt);
    void currentOutputChanged(double arg);
    void resetted();

public slots:
    void step(double dt, bool parentEnabled);
    void fire();
    void receiveFire(NodeEngine *sender);
    void receiveCurrent(double currentOutput, NodeEngine *sender);
    void finalizeStep(double dt);
    void setCurrentOutput(double arg);

protected:
    virtual void stepEvent(double dt, bool parentEnabled);
    virtual void fireEvent();
    virtual void receiveFireEvent(NodeEngine *sender);
    virtual void receiveCurrentEvent(double currentOutput, NodeEngine *sender);
    virtual void finalizeStepEvent(double dt);
    virtual void resetEvent();

private:
    bool m_hasFired = false;
    double m_currentOutput = 0.0;
    bool m_inhibitory = false;
};

#endif // NODEENGINE_H
