#include "edge.h"

#include "nodebase.h"

Edge::Edge(QQuickItem *parent)
    : QQuickItem(parent)
{
}

Edge::~Edge()
{
    m_itemA = nullptr;
    m_itemB = nullptr;
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
        previousItemA->removeEdge(this);
    }

    m_itemA = arg;

    if(m_itemA) {
        m_itemA->addEdge(this);
    }

    emit itemAChanged(arg);
}

void Edge::setItemB(NodeBase *arg)
{
    if (m_itemB == arg)
        return;

    NodeBase* previousItemB = m_itemB;
    if(previousItemB) {
        previousItemB->removeEdge(this);
    }

    m_itemB = arg;

    if(m_itemB) {
        m_itemB->addEdge(this);
    }

    emit itemBChanged(arg);
}

void Edge::clear()
{
    setItemA(nullptr);
    setItemB(nullptr);
}

