#include "nodebase.h"

NodeBase::NodeBase(QQuickItem *parent)
    : QQuickItem(parent)
{

}

NodeBase::~NodeBase()
{

}

NodeEngine *NodeBase::engine() const
{
    return m_engine;
}

void NodeBase::setEngine(NodeEngine *arg)
{
    if (m_engine == arg)
        return;

    m_engine = arg;
    emit engineChanged(arg);
}
