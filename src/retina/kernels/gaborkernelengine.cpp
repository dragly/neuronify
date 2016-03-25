#include "gaborkernelengine.h"

GaborKernelEngine::GaborKernelEngine()
{
}

double GaborKernelEngine::advance(int idx, int idy)
{
    double sigmaX = 5.;
    double sigmaY = 5.;
    double k = 0.5;
    double phi = 0.0;
    double theta = 0.0;

    double x =  (idx - m_resolutionHeight/2) * cos(theta)
            + (idy-m_resolutionWidth/2) * sin(theta);
    double y = -(idx - m_resolutionHeight/2) * sin(theta)
            + (idy-m_resolutionWidth/2) * cos(theta);

    double prefactor = 1.0/(2.* pi * sigmaX * sigmaY);
    double expFactor = exp(-x*x/(2.* sigmaX * sigmaX) - y*y/(2. * sigmaY * sigmaY));
    double cosFactor = cos(k * x - phi);
    return prefactor * expFactor * cosFactor;
}


void GaborKernelEngine::createKernel(vector<vector<double> >* spatial)
{

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            spatial->at(i).at(j)= advance(i,j)*100.;
//            qDebug() << m_kernel.at(i).at(j);
        }
    }

//    for(int i=0; i<m_resolutionHeight; i++)    //This loops on the rows.
//    {
//        for(int j=0; j<m_resolutionWidth; j++) //This loops on the columns
//        {
//            cout << setprecision(3)<< fixed << m_kernel[i][j]  << "  ";
//        }
//        cou
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
