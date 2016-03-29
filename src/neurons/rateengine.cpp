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

    m_spikeTimes.push_back(m_time);
}

void RateEngine::computeFiringRate()
{
    double value = 0.0;
    double sigma = 100e-3;

    for(int j= m_spikeTimes.size(); j > 0; j--){
        double tau = m_time - m_spikeTimes[j-1];
        if(tau < m_windowDuration){
            value += 1./(sqrt(2*sigma*sigma)) *exp(-tau*tau/2./sigma/sigma);

        //value += MathHelper::heaviside(
        //sigma * sigma * tau * exp(-sigma * tau));

        }else{
            break;
        }
    }
    m_firingRate = value;
    emit firingRateChanged(m_firingRate);
}


void RateEngine::stepEvent(double dt)
{
    if(m_neuronCount < 1){
        return;
    }
    m_time +=dt;
    computeFiringRate();
}


double RateEngine::firingRate() const
{
    return m_firingRate;
}

double RateEngine::windowDuration() const
{
    return m_windowDuration;
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

void RateEngine::setWindowDuration(double windowDuration)
{
    if (m_windowDuration == windowDuration)
        return;

    m_windowDuration = windowDuration;
    emit windowDurationChanged(windowDuration);
}

void RateEngine::setNeuronCount(int neuronCount)
{
    if (m_neuronCount == neuronCount)
        return;

    m_neuronCount = neuronCount;
    emit neuronCountChanged(neuronCount);
}

