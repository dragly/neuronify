#include "graphengine.h"

#include "nodebase.h"
#include "edge.h"
#include "nodeengine.h"

GraphEngine::GraphEngine(QQuickItem *parent)
    : QQuickItem(parent)
{

}

GraphEngine::~GraphEngine()
{

}

QQmlListProperty<NodeBase> GraphEngine::nodes()
{
    return QQmlListProperty<NodeBase>(this, m_nodes);
}

QQmlListProperty<Edge> GraphEngine::edges()
{
    return QQmlListProperty<Edge>(this, m_edges);
}

void GraphEngine::addNode(NodeBase *node)
{
    m_nodes.append(node);
}

void GraphEngine::addEdge(Edge *edge)
{
    m_edges.append(edge);
}

void GraphEngine::step(double dt)
{
    for(NodeBase* node : m_nodes) {
        if(node->engine()) {
            node->engine()->step(dt);
        }
    }

    for(Edge* edge : m_edges) {
        if(edge->itemA() && edge->itemB()) {
            NodeEngine* engineA = edge->itemA()->engine();
            NodeEngine* engineB = edge->itemB()->engine();
            if(engineA->hasFired()) {
                engineB->stimulate(engineA->stimulation());
            }
        }
    }

    for(NodeBase* node : m_nodes) {
        node->engine()->finalizeStep(dt);
    }
}

