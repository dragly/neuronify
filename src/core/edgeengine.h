#ifndef EDGEENGINE_H
#define EDGEENGINE_H

#include <QQuickItem>

#include "neuronifyobject.h"

class NodeEngine;

class EdgeEngine : public NeuronifyObject
{
    Q_OBJECT
    Q_PROPERTY(double currentOutput READ currentOutput WRITE setCurrentOutput NOTIFY currentOutputChanged)

public:
    explicit EdgeEngine(QQuickItem *parent = 0);

    double currentOutput() const;

signals:
    void stepped(double dt, bool parentEnabled);
    void currentOutputChanged(double currentOutput);
    void receivedFire(NodeEngine *sender);

public slots:
    void step(double dt, bool parentEnabled);
    void setCurrentOutput(double currentOutput);
    void receiveFire(NodeEngine *sender);

protected:
    virtual void stepEvent(double dt, bool parentEnabled);
    virtual void receiveFireEvent(NodeEngine *sender);

private:
    double m_currentOutput = 0.;
};

#endif // EDGEENGINE_H
