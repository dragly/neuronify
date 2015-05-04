#include "current.h"

#include "neuronengine.h"

Current::Current(QQuickItem *parent)
    : NodeEngine(parent)
{
}

Current::~Current()
{
}

double Current::current() const
{
    return m_current;
}

void Current::setCurrent(double arg)
{
    if (m_current == arg)
        return;

    m_current = arg;
    emit currentChanged(arg);
}

