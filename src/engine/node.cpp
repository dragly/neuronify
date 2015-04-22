#include "node.h"

Node::Node(QQuickItem *parent)
    : Entity(parent)
{

}

Node::~Node()
{

}

double Node::stimulation() const
{
    return m_stimulation;
}

void Node::outputConnectionStep(Node *target)
{
    if(hasFired()) {
        target->stimulate(m_stimulation);
    }
}

void Node::inputConnectionStep(Node *source)
{

}

void Node::setStimulation(double arg)
{
    if (m_stimulation == arg)
        return;

    m_stimulation = arg;
    emit stimulationChanged(arg);
}

