#include "edge.h"

#include "nodebase.h"

/*!
 * \class Edge
 * \inmodule Neuronify
 * \ingroup neuronify-core
 * \brief The Edge class is a basic connection between two nodes.
 *
 * When two objects of the NodeBase type are to be connected in the GraphEngine
 * an Edge object is used to hold a reference to the two objects.
 *
 * It is the responsibility of the Edge object to notify the two NodeBase
 * objects about when it has been added or removed.
 *
 */

Edge::Edge(QQuickItem *parent)
    : QQuickItem(parent)
{
    m_curved = 0;
}

NodeBase *Edge::itemA() const
{
    return m_itemA;
}

NodeBase *Edge::itemB() const
{
    return m_itemB;
}

int Edge::curved() const
{
    return m_curved;
}

void Edge::setItemA(NodeBase *arg)
{
    if (m_itemA == arg)
        return;

    NodeBase* previousItemA = m_itemA;
    if(previousItemA) {
        emit previousItemA->edgeRemoved(this);
    }

    m_itemA = arg;

    if(m_itemA) {
        emit m_itemA->edgeAdded(this);
    }

    emit itemAChanged(arg);
}

void Edge::setItemB(NodeBase *arg)
{
    if (m_itemB == arg)
        return;

    NodeBase* previousItemB = m_itemB;
    if(previousItemB) {
        emit previousItemB->edgeRemoved(this);
    }

    m_itemB = arg;

    if(m_itemB) {
        emit m_itemB->edgeAdded(this);
    }

    emit itemBChanged(arg);
}

void Edge::setCurved(int curved)
{
    if (m_curved == curved)
        return;

    m_curved = curved;
    emit curvedChanged(curved);
}

