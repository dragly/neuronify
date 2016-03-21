#include "rectangularkernelengine.h"

RectangularKernelEngine::RectangularKernelEngine()
{

}


void RectangularKernelEngine::createKernel(vector<vector<double> > *spatial)
{

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            spatial->at(i).at(j) = 127;
        }
    }

    for(int i = 0; i < m_resolutionWidth/2; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            spatial->at(i).at(j) = -128;
        }
    }
}
