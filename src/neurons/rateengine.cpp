#include "rateengine.h"

#include <QDebug>
#include <cmath>

#include "current.h"

using namespace std;

/*!
 * \class RateEngine
 * \inmodule Neuronify
 * \ingroup neuronify-neurons
 * \brief The RateEngine class is a common engine used RatePlot.
 */


RateEngine::RateEngine(QQuickItem *parent)
    : NodeEngine(parent)
{

}

void RateEngine::receiveFireEvent(double fireOutput, NodeEngine *sender)
{
    Q_UNUSED(sender);
    Q_UNUSED(fireOutput);

    m_spikeCount +=1;


}

double RateEngine::firingRate() const
{
    return m_firingRate;
}

double RateEngine::binLength() const
{
    return m_binLength;
}

int RateEngine::neuronCount() const
{
    return m_neuronCount;
}

void RateEngine::setFiringRate(double firingRate)
{
    if (m_firingRate == firingRate)
        return;

    m_firingRate = firingRate;
    emit firingRateChanged(firingRate);
}

void RateEngine::setBinLength(double binLength)
{
    if (m_binLength == binLength)
        return;

    m_binLength = binLength;
    emit binLengthChanged(binLength);
}

void RateEngine::setNeuronCount(int neuronCount)
{
    if (m_neuronCount == neuronCount)
        return;

    m_neuronCount = neuronCount;
    emit neuronCountChanged(neuronCount);
}


void RateEngine::stepEvent(double dt)
{
    if(m_neuronCount < 1){
        return;
    }

    m_window +=dt;
//    qDebug() << m_window << " F " << m_firingRate << "  S " << m_spikeCount
//             << "  N " << m_neuronCount;
    if(m_window > m_binLength){
        m_firingRate = m_spikeCount / m_window/ m_neuronCount;
//        m_firingRate = m_firingRate * 0.9 + 0.1 * m_spikeCount / m_window/m_neuronCount;
        m_spikeCount = 0;
        m_window = 0.0;
        emit firingRateChanged(m_firingRate);
    }
}
