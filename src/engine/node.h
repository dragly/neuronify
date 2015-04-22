#ifndef NODE_H
#define NODE_H

#include <QQuickItem>

#include "entity.h"

class Edge;
class Node : public Entity
{
    Q_OBJECT
    Q_PROPERTY(double stimulation READ stimulation WRITE setStimulation NOTIFY stimulationChanged)

public:
    explicit Node(QQuickItem* parent = 0);
    ~Node();

    double stimulation() const;

signals:
    void edgeAdded(Edge* edge);
    void edgeRemoved(Edge* edge);

    void stimulationChanged(double arg);

public slots:
    void outputConnectionStep(Node* target);
    void inputConnectionStep(Node* source);

    void setStimulation(double arg);

private:

    double m_stimulation;
};

#endif // NODE_H
