#include "graphengine.h"

#include "node.h"
#include "edge.h"

GraphEngine::GraphEngine(QQuickItem *parent)
    : QQuickItem(parent)
{

}

GraphEngine::~GraphEngine()
{

}

QQmlListProperty<Node> GraphEngine::nodes()
{
    return QQmlListProperty<Node>(this, m_nodes);
}

QQmlListProperty<Edge> GraphEngine::edges()
{
    return QQmlListProperty<Edge>(this, m_edges);
}

void GraphEngine::addNode(Node *node)
{
    m_nodes.append(node);
}

void GraphEngine::addEdge(Edge *edge)
{
    m_edges.append(edge);
}

void GraphEngine::step(double dt)
{
    int counter = 0;
    for(Node* node : m_nodes) {
        node->step(dt);
        counter++;
    }

    for(Edge* edge : m_edges) {
        if(edge->itemA() && edge->itemB()) {
            edge->itemA()->outputConnectionStep(edge->itemB());
            edge->itemB()->inputConnectionStep(edge->itemA());
        }
    }

    for(Node* node : m_nodes) {
        node->finalizeStep(dt);
    }
}

