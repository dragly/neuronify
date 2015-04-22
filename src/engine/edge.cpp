#include "edge.h"

#include "node.h"

Edge::Edge(QQuickItem *parent)
    : QQuickItem(parent)
{

}

Edge::~Edge()
{

}

Node *Edge::itemA() const
{
    return m_itemA;
}

Node *Edge::itemB() const
{
    return m_itemB;
}

void Edge::setItemA(Node *arg)
{
    if (m_itemA == arg)
        return;

    Node* previousItemA = m_itemA;
    if(previousItemA) {
        previousItemA->edgeRemoved(this);
    }

    m_itemA = arg;

    m_itemA->edgeAdded(this);

    emit itemAChanged(arg);
}

void Edge::setItemB(Node *arg)
{
    if (m_itemB == arg)
        return;

    Node* previousItemA = m_itemB;
    if(previousItemA) {
        previousItemA->edgeRemoved(this);
    }

    m_itemB = arg;

    m_itemB->edgeAdded(this);

    emit itemBChanged(arg);
}

