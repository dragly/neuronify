#include "graphengine.h"

#include "nodebase.h"
#include "edgebase.h"
#include "nodeengine.h"
#include "edgeengine.h"

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

QQmlListProperty<EdgeBase> GraphEngine::edges()
{
    return QQmlListProperty<EdgeBase>(this, m_edges);
}

int GraphEngine::nodeIndex(NodeBase *node) const
{
    return m_nodes.indexOf(node);
}

void GraphEngine::addNode(NodeBase *node)
{
    m_nodes.append(node);
}

void GraphEngine::addEdge(EdgeBase *edge)
{
    for (EdgeBase *otherEdge : m_edges) {
        if (edge->itemA() == otherEdge->itemB() && edge->itemB() == otherEdge->itemA()){
            edge->setCurved(-1);
            otherEdge->setCurved(1);
        }
    }

    m_edges.append(edge);


}

void GraphEngine::removeNode(NodeBase *node)
{
    m_nodes.removeAll(node);
    QList<EdgeBase*> toDelete;
    for(EdgeBase *edge : m_edges) {
        if(edge->itemA() == node || edge->itemB() == node) {
            toDelete.append(edge);
        }
    }
    for(EdgeBase *edge : toDelete) {
        removeEdge(edge);
    }
    node->deleteLater();
}

void GraphEngine::removeEdge(EdgeBase *edge)
{
    for (EdgeBase *otherEdge : m_edges) {
        if (edge->itemA() == otherEdge->itemB() && edge->itemB() == otherEdge->itemA()){
            edge->setCurved(0);
            otherEdge->setCurved(0);
        }
    }

    if(edge->itemA()) {
        emit edge->itemA()->edgeRemoved(edge);
    }
    if(edge->itemB()) {
        emit edge->itemB()->edgeRemoved(edge);
    }

    if(edge->itemA()) {
        edge->m_itemA = nullptr;
        emit edge->itemAChanged(nullptr);
    }
    if(edge->itemB()) {
        edge->m_itemB = nullptr;
        emit edge->itemBChanged(nullptr);
    }
    m_edges.removeAll(edge);
    edge->deleteLater();
}

void GraphEngine::step(double dt)
{
    // step all nodes
    for(NodeBase* node : m_nodes) {
        if(node->engine()) {
            node->engine()->step(dt, true);
        }
    }

    for(EdgeBase* edge : m_edges) {
        if(edge->engine()){
            edge->engine()->step(dt, true);
        }
    }

    //communicate events between nodes
    for(EdgeBase* edge : m_edges) {
        if(!(edge->itemA()) || !(edge->itemB())) {
            continue;
        }
        NodeEngine* engineA = edge->itemA()->engine();
        NodeEngine* engineB = edge->itemB()->engine();
        if(!engineA || !engineB) {
            continue;
        }
        if(engineA->hasFired()) {
            if(edge->engine()) {
                edge->engine()->receiveFire(engineA);
            }
            engineB->receiveFire(engineA);
        }

        double sign = 1.0;
        if(edge->itemA()->inhibitory()) {
            sign = -1.0;
        }

        if(edge->engine()) {
            if(edge->engine()->currentOutput() != 0.0) {
                engineB->receiveCurrent(sign * edge->engine()->currentOutput(), engineA);
            }
        }

        //TODO: do we need this with the new synapses?
        if(engineA->currentOutput() != 0.0) {
            engineB->receiveCurrent(sign * engineA->currentOutput(), engineA);
        }
    }

    //finalize step
    // TODO: remove this if not needed
    for(NodeBase* node : m_nodes) {
        if(node->engine()) {
            node->engine()->finalizeStep(dt);
        }
    }
}

