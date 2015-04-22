#include "edge.h"

#include "nodebase.h"

Edge::Edge(QQuickItem *parent)
    : QQuickItem(parent)
{

}

Edge::~Edge()
{

}

NodeBase *Edge::itemA() const
{
    return m_itemA;
}

NodeBase *Edge::itemB() const
{
    return m_itemB;
}

void Edge::setItemA(NodeBase *arg)
{
    if (m_itemA == arg)
        return;

    NodeBase* previousItemA = m_itemA;
    if(previousItemA) {
        previousItemA->edgeRemoved(this);
    }

    m_itemA = arg;

    m_itemA->edgeAdded(this);

    emit itemAChanged(arg);
}

void Edge::setItemB(NodeBase *arg)
{
    if (m_itemB == arg)
        return;

    NodeBase* previousItemA = m_itemB;
    if(previousItemA) {
        previousItemA->edgeRemoved(this);
    }

    m_itemB = arg;

    m_itemB->edgeAdded(this);

    emit itemBChanged(arg);
}

