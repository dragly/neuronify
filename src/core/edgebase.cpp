#include "edgebase.h"
#include "edgeengine.h"
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

EdgeBase::EdgeBase(QQuickItem *parent)
    : NeuronifyObject(parent)
{
    m_curved = 0;
}

NodeBase *EdgeBase::itemA() const
{
    return m_itemA;
}

NodeBase *EdgeBase::itemB() const
{
    return m_itemB;
}

int EdgeBase::curved() const
{
    return m_curved;
}

EdgeEngine *EdgeBase::engine() const
{
    return m_engine;
}

void EdgeBase::setItemA(NodeBase *arg)
{
    if (m_itemA == arg)
        return;

    NodeBase* previousItemA = m_itemA;
    if(previousItemA) {
        emit previousItemA->edgeRemoved(this);
    }

    m_itemA = arg;

    emit itemAChanged(arg);
}

void EdgeBase::setItemB(NodeBase *arg)
{
    if (m_itemB == arg)
        return;

    NodeBase* previousItemB = m_itemB;
    if(previousItemB) {
        emit previousItemB->edgeRemoved(this);
    }

    m_itemB = arg;

    emit itemBChanged(arg);
}

void EdgeBase::setCurved(int curved)
{
    if (m_curved == curved)
        return;

    m_curved = curved;
    emit curvedChanged(curved);
}

void EdgeBase::setEngine(EdgeEngine *engine)
{
    if (m_engine == engine)
        return;

    m_engine = engine;
    emit engineChanged(engine);
}

