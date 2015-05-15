#include "graphengine.h"

#include "nodebase.h"
#include "edge.h"
#include "nodeengine.h"

/*!
 * \class GraphEngine
 * \inmodule Neuronify
 * \ingroup neuronify-core
 * \brief The GraphEngine class is the core engine of Neuronify
 *
 * GraphEngine holds all the nodes and edges of the network.
 * It iterates the network forward in time by stepping each node and
 * organizing the communication along edges.
 *
 * \sa NodeBase, NodeEngine, Node
 */

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

int GraphEngine::nodeIndex(NodeBase *node) const
{
    return m_nodes.indexOf(node);
}

void GraphEngine::addNode(NodeBase *node)
{
    m_nodes.append(node);
}

void GraphEngine::addEdge(Edge *edge)
{
    m_edges.append(edge);
}

void GraphEngine::removeNode(NodeBase *node)
{
    m_nodes.removeAll(node);
    QList<Edge*> toDelete;
    for(Edge *edge : m_edges) {
        if(edge->itemA() == node || edge->itemB() == node) {
            edge->clear();
            toDelete.append(edge);
        }
    }
    for(Edge *edge : toDelete) {
        edge->deleteLater();
    }
}

void GraphEngine::removeEdge(Edge *edge)
{
    if(edge->itemA()) {
        edge->setItemA(nullptr); // This also removes edges from itemA
    }
    if(edge->itemB()) {
        edge->setItemB(nullptr); // This also removes edges from itemB
    }
    m_edges.removeAll(edge);
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
                engineB->receiveFire(engineA->fireOutput());
            }
            if(engineA->currentOutput() != 0.0) {
                engineB->receiveCurrent(engineA->currentOutput());
            }
        }
    }

    for(NodeBase* node : m_nodes) {
        node->engine()->finalizeStep(dt);
    }
}

