#include "dogkernelengine.h"

DogKernelEngine::DogKernelEngine()
{

}


void DogKernelEngine::createKernel(vector<vector<double> > *spatial)
{

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            spatial->at(i).at(j)= advance(i,j);
        }
    }

//    cout << spatial->at(m_resolutionWidth/2).at(m_resolutionHeight/2) << endl;

}


double DogKernelEngine::advance(int idx, int idy)
{
    double A = 1.0;
    double a = 2.25;
    double B = 0.85;
    double b = 0.83;


    double x =  (idx - m_resolutionHeight/2);
    double y =  (idy - m_resolutionWidth/2);

    double r2 = sqrt(x*x + y*y);
    double center   = A / (a*a) / pi * exp(-r2 / (a*a));
    double surround = B / (b*b) / pi * exp(-r2 / (b*b));


    return center - surround;
}
