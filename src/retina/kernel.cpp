#include "kernel.h"

#include <cmath>

/*!
 * \class Kernel
 * \inmodule Neuronify
 * \ingroup neuronify-sensors
 * \brief Creates different kernel types.
 */


Kernel::Kernel(QQuickItem *parent)
    : NeuronifyObject(parent)
{
}


void Kernel::recreate()
{
    m_spatial.resize(m_resolutionWidth);
    for(int i = 0; i < m_resolutionWidth; i++){
        m_spatial.at(i).resize(m_resolutionHeight,0);
    }

    if(m_abstractKernelEngineType == nullptr){
        return;
    }
    m_abstractKernelEngineType->createKernel(&m_spatial);
    m_spatialImage = QImage(m_resolutionWidth,
                            m_resolutionHeight,
                            QImage::Format_RGBA8888);

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            int gray = (m_spatial.at(i).at(j) + 1) * 127;
            QRgb color = qRgba(gray, gray, gray, m_imageAlpha);
            m_spatialImage.setPixel(i,j,color);

        }
    }

}


int Kernel::resolutionHeight() const
{
    return m_resolutionHeight;
}

int Kernel::resolutionWidth() const
{
    return m_resolutionWidth;
}

vector<vector<double> > Kernel::spatial()
{
    if(m_spatial.empty()){
        recreate();
    }
    return m_spatial;
}


QImage Kernel::spatialImage() const
{
    return m_spatialImage;
}

int Kernel::imageAlpha() const
{
    return m_imageAlpha;
}



void Kernel::setResolutionHeight(int resolutionHeight)
{
    if (m_resolutionHeight == resolutionHeight)
        return;

    m_resolutionHeight = resolutionHeight;
    recreate();
    emit resolutionHeightChanged(resolutionHeight);
}

void Kernel::setResolutionWidth(int resolutionWidth)
{
    if (m_resolutionWidth == resolutionWidth)
        return;

    m_resolutionWidth = resolutionWidth;
    recreate();
    emit resolutionWidthChanged(resolutionWidth);
}

void Kernel::setSpatialImage(QImage image)
{
    if (m_spatialImage == image)
        return;

    m_spatialImage = image;
    emit imageChanged(image);
}




void Kernel::setAbstractKernelEngineType(AbstractKernelEngine* abstractKernelEngineType)
{
    if (m_abstractKernelEngineType == abstractKernelEngineType)
        return;

    if(m_abstractKernelEngineType){
        disconnect(m_abstractKernelEngineType, 0, this, 0);
    }

    if(abstractKernelEngineType){
        connect(abstractKernelEngineType, &AbstractKernelEngine::needsRecreation, this, &Kernel::recreate);
    }

    m_abstractKernelEngineType = abstractKernelEngineType;
    recreate();
    emit abstractKernelEngineTypeChanged(abstractKernelEngineType);
}

void Kernel::setImageAlpha(int imageAlpha)
{
    if (m_imageAlpha == imageAlpha)
        return;

    m_imageAlpha = imageAlpha;
    emit imageAlphaChanged(imageAlpha);
}



AbstractKernelEngine* Kernel::abstractKernelEngineType() const
{
    return m_abstractKernelEngineType;
}




