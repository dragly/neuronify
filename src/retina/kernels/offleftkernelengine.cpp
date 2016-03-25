#include "offleftkernelengine.h"

OffLeftKernelEngine::OffLeftKernelEngine()
{

}


void OffLeftKernelEngine::createKernel(vector<vector<double> > *spatial)
{
    for(int i = 0; i < m_resolutionWidth/2; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            spatial->at(i).at(j) = -1;
        }
    }

    for(int i = m_resolutionWidth/2; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            spatial->at(i).at(j) = 1;
        }
    }

}
