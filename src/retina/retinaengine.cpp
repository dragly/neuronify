#include "retinaengine.h"

#include <QPainter>
#include <QVideoRendererControl>
#include <random>

/*!
 * \class RetinaEngine
 * \inmodule Neuronify
 * \ingroup neuronify-sensors
 * \brief Calculates the firing rate of a neuron based on the stimulus and the receptive field type.
 */


RetinaEngine::RetinaEngine()
{
}

RetinaEngine::~RetinaEngine()
{
}

void RetinaEngine::receivedImage()
{
    if(!m_kernel){
        return;
    }

    int resolutionHeight = m_kernel->resolutionHeight();
    int resolutionWidth = m_kernel->resolutionWidth();
    m_stim.resize(resolutionWidth);
    for(int i = 0; i < resolutionWidth; i++){
        m_stim.at(i).resize(resolutionHeight,0.0);
    }

    m_paintedImage =  m_videoSurface->paintedImage();
    m_paintedImage =  m_paintedImage.scaled(resolutionWidth,resolutionHeight);

    for(int i = 0; i < m_paintedImage.width(); i++){
        for(int j = 0; j < m_paintedImage.height(); j++){
#ifdef Q_OS_ANDROID
            int gray = m_paintedImage.pixel(i,j);
#else
            int gray = qGray(m_paintedImage.pixel(i,j));
            QRgb color = qRgb(gray, gray, gray);
            m_paintedImage.setPixel(i,j,color);
#endif
            m_stim.at(i).at(j) = gray-128./256;
        }
    }

    calculateFiringRate();
    update();
}

void RetinaEngine::calculateFiringRate()
{
    if(!m_kernel){
        return;
    }

    vector< vector <double>> spatial = m_kernel->spatial();
    int resolutionHeight= m_kernel->resolutionHeight();
    int resolutionWidth = m_kernel->resolutionWidth();

    m_firingRate = 0.0;
    for(int i = 0; i < resolutionWidth; i++){
        for(int j = 0; j < resolutionHeight; j++){
            m_firingRate += m_stim.at(i).at(j) *  spatial.at(i).at(j)/256;
        }
    }
    int size = resolutionHeight*resolutionWidth;
    m_firingRate /= size;
    if(m_firingRate < 0){
        m_firingRate = 0;
    }
}



void RetinaEngine::stepEvent(double dt)
{
    std::random_device rd;
    std::mt19937 gen(rd());
    std::uniform_real_distribution<> dis(0,1);

    double shouldFire = (dis(gen) < m_firingRate*dt);
    if(shouldFire){
        fire();
    }


}


void RetinaEngine::setVideoSurface(VideoSurface *videoSurface)
{

    if (m_videoSurface == videoSurface)
        return;
    if(m_videoSurface){
        disconnect(m_videoSurface, &VideoSurface::gotImage, this, &RetinaEngine::receivedImage);
    }
    m_videoSurface = videoSurface;
    if(m_videoSurface){
        connect(m_videoSurface, &VideoSurface::gotImage, this, &RetinaEngine::receivedImage);
    }
    emit videoSurfaceChanged(videoSurface);
}





void RetinaEngine::setKernel(Kernel *kernel)
{
    if (m_kernel == kernel)
        return;

    m_kernel = kernel;



    emit kernelChanged(kernel);
}


QImage RetinaEngine::paintedImage() const
{
    return m_paintedImage;
}




Kernel *RetinaEngine::kernel() const
{
    return m_kernel;
}

VideoSurface *RetinaEngine::videoSurface() const
{
    return m_videoSurface;
}








