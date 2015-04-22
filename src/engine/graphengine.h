#ifndef GRAPHENGINE_H
#define GRAPHENGINE_H

#include <QQuickItem>

class Node;
class Edge;
class GraphEngine : public QQuickItem
{
    Q_OBJECT
    Q_PROPERTY(QQmlListProperty<Node> nodes READ nodes)
    Q_PROPERTY(QQmlListProperty<Edge> edges READ edges)
public:
    explicit GraphEngine(QQuickItem* parent = 0);
    ~GraphEngine();

    QQmlListProperty<Node> nodes();
    QQmlListProperty<Edge> edges();

public slots:
    void step(double dt);
    void addNode(Node* node);
    void addEdge(Edge* edge);

private:
    QList<Node*> m_nodes;
    QList<Edge*> m_edges;
};

#endif // GRAPHENGINE_H
