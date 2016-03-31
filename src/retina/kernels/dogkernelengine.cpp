#include "dogkernelengine.h"

DogKernelEngine::DogKernelEngine()
{
    m_centerWeight = 1.0;
    m_centerExp = 100.;
    m_surroundWeight = 0.3;
    m_surroundExp = 20.;
}


void DogKernelEngine::createKernel(vector<vector<double> > *spatial)
{
    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            spatial->at(i).at(j)= advance(m_x.at(i), m_y.at(j))
                    * pow(-1, int(m_isOffCenter));
        }
    }
}

bool DogKernelEngine::isOffCenter() const
{
    return m_isOffCenter;
}


double DogKernelEngine::advance(double x, double y)
{
    double r2 = x*x + y*y;
    double center   = m_centerWeight * exp(-m_centerExp * r2);
    double surround = m_surroundWeight * exp(-m_surroundExp * r2);
    return center - surround;
}


double DogKernelEngine::centerWeight() const
{
    return m_centerWeight;
}

double DogKernelEngine::centerExp() const
{
    return m_centerExp;
}

double DogKernelEngine::surroundWeight() const
{
    return m_surroundWeight;
}

double DogKernelEngine::surroundExp() const
{
    return m_surroundExp;
}

void DogKernelEngine::setCenterWeight(double centerWeight)
{
    if (m_centerWeight == centerWeight)
        return;

    m_centerWeight = centerWeight;
    emit centerWeightChanged(centerWeight);
}

void DogKernelEngine::setCenterExp(double centerExp)
{
    if (m_centerExp == centerExp)
        return;

    m_centerExp = centerExp;
    emit centerExpChanged(centerExp);
}

void DogKernelEngine::setSurroundWeight(double surroundWeight)
{
    if (m_surroundWeight == surroundWeight)
        return;

    m_surroundWeight = surroundWeight;
    emit surroundWeightChanged(surroundWeight);
}

void DogKernelEngine::setSurroundExp(double surroundExp)
{
    if (m_surroundExp == surroundExp)
        return;

    m_surroundExp = surroundExp;
    emit surroundExpChanged(surroundExp);
}

void DogKernelEngine::setIsOffCenter(bool isOffCenter)
{
    if (m_isOffCenter == isOffCenter)
        return;

    m_isOffCenter = isOffCenter;
    emit isOffCenterChanged(isOffCenter);
    emit needsRecreation();
}

