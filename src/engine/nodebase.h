#ifndef NODEBASE_H
#define NODEBASE_H

#include <QQuickItem>

#include "entity.h"

class NodeEngine;
class Edge;
class NodeBase : public QQuickItem
{
    Q_OBJECT
    Q_PROPERTY(NodeEngine* engine READ engine WRITE setEngine NOTIFY engineChanged)

public:
    explicit NodeBase(QQuickItem* parent = 0);
    ~NodeBase();

    NodeEngine* engine() const;

signals:
    void edgeAdded(Edge* edge);
    void edgeRemoved(Edge* edge);
    void engineChanged(NodeEngine* arg);

public slots:
    void setEngine(NodeEngine* arg);

private:
    NodeEngine* m_engine;
    double m_stimulation;
};

#endif // NODEBASE_H
