#include "offbottomkernelengine.h"

OffBottomKernelEngine::OffBottomKernelEngine()
{

}

void OffBottomKernelEngine::createKernel(vector<vector<double> > *spatial)
{

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            spatial->at(i).at(j)= -128;
        }
    }

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight/2; j++){
            spatial->at(i).at(j) = 127;
        }
    }
}
