#include "dogkernelengine.h"

DogKernelEngine::DogKernelEngine()
{

}


void DogKernelEngine::createKernel(vector<vector<double> > *spatial)
{

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            spatial->at(i).at(j)= advance(m_x.at(i), m_y.at(j));
        }
    }
}


double DogKernelEngine::advance(double x, double y)
{
    double A = 1.0;
    double a = 100.;
    double B = 0.3;
    double b = 20.;

    double r2 = x*x + y*y;
    double center   = A * exp(-a * r2);
    double surround = B * exp(-b * r2);


    return center - surround;
}
