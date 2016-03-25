#include "gaborkernelengine.h"

GaborKernelEngine::GaborKernelEngine()
{
}

double GaborKernelEngine::advance(double x, double y)
{
    double sigmaX = 0.1;
    double sigmaY = 0.2;
    double k = 20.;
    double phi = 0.0;
    double theta = 0.0;

    double xr =  x * cos(theta) + y * sin(theta);
    double yr = -x * sin(theta) + y * cos(theta);

    double prefactor = 1.0/(2.* pi * sigmaX * sigmaY);
    double expFactor = exp(-xr*xr/(2.* sigmaX * sigmaX)
                           - yr*yr/(2. * sigmaY * sigmaY));
    double cosFactor = cos(k * xr - phi);
    return 1 * expFactor * cosFactor;
}


void GaborKernelEngine::createKernel(vector<vector<double> >* spatial)
{

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            spatial->at(i).at(j)= advance(m_x.at(i), m_y.at(j));
        }
    }
}

double GaborKernelEngine::sigmaX() const
{
    return m_sigmaX;
}

double GaborKernelEngine::sigmaY() const
{
    return m_sigmaY;
}

double GaborKernelEngine::k() const
{
    return m_k;
}

double GaborKernelEngine::phi() const
{
    return m_phi;
}

double GaborKernelEngine::theta() const
{
    return m_theta;
}

void GaborKernelEngine::setSigmaX(double sigmaX)
{
    if (m_sigmaX == sigmaX)
        return;

    m_sigmaX = sigmaX;
    emit sigmaXChanged(sigmaX);
}

void GaborKernelEngine::setSigmaY(double sigmaY)
{
    if (m_sigmaY == sigmaY)
        return;

    m_sigmaY = sigmaY;
    emit sigmaYChanged(sigmaY);
}

void GaborKernelEngine::setK(double k)
{
    if (m_k == k)
        return;

    m_k = k;
    emit kChanged(k);
}

void GaborKernelEngine::setPhi(double phi)
{
    if (m_phi == phi)
        return;

    m_phi = phi;
    emit phiChanged(phi);
}

void GaborKernelEngine::setTheta(double theta)
{
    if (m_theta == theta)
        return;

    m_theta = theta;
    emit thetaChanged(theta);
}
