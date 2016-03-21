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
//    cout << m_resolutionWidth << endl;
//    cout << m_resolutionHeight << endl;

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            spatial->at(i).at(j)= advance(i,j)*20000.;
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
