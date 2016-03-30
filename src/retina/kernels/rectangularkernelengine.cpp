#include "rectangularkernelengine.h"

RectangularKernelEngine::RectangularKernelEngine()
{

}


void RectangularKernelEngine::createKernel(vector<vector<double> > *spatial)
{
    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            double y = m_x.at(i) * sin(m_orientation)
                    + m_y.at(j) * cos(m_orientation);
            spatial->at(i).at(j) = advance(y);
        }
    }

}

double RectangularKernelEngine::advance(double y)
{
    if(y < 0){
        return -1;
    }else{
        return 1;
    }
}


double RectangularKernelEngine::orientation() const
{
    return m_orientation;
}

void RectangularKernelEngine::setOrientation(double orientation)
{
    if (m_orientation == orientation)
        return;

    m_orientation = orientation;
    emit orientationChanged(orientation);
    emit needsRecreation();
}

