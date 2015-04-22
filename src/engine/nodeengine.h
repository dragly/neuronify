#ifndef NODEENGINE_H
#define NODEENGINE_H

#include <QQuickItem>

#include "entity.h"

class NodeEngine : public Entity
{
    Q_OBJECT
    Q_PROPERTY(double stimulation READ stimulation WRITE setStimulation NOTIFY stimulationChanged)

    double m_stimulation;

public:
    explicit NodeEngine(QQuickItem *parent = 0);
    ~NodeEngine();

    double stimulation() const;

signals:
    void stimulationChanged(double arg);

public slots:
    void setStimulation(double arg);
};

#endif // NODEENGINE_H
