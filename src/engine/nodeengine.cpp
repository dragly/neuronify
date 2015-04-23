#include "nodeengine.h"

NodeEngine::NodeEngine(QQuickItem *parent)
    : EngineBase(parent)
{

}

NodeEngine::~NodeEngine()
{

}

double NodeEngine::stimulation() const
{
    return m_stimulation;
}

void NodeEngine::setStimulation(double arg)
{
    if (m_stimulation == arg)
        return;

    m_stimulation = arg;
    emit stimulationChanged(arg);
}

