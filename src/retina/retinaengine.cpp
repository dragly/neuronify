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
    m_paintedImage =  m_paintedImage.scaled(resolutionWidth,
                                            resolutionHeight);

    for(int i = 0; i < m_paintedImage.width(); i++){
        for(int j = 0; j < m_paintedImage.height(); j++){
            int value = 0;
            value = qGray(m_paintedImage.pixel(i,j));
#ifdef Q_OS_ANDROID
            // already RGB with grey color on Android, can pick only one component, but let's not for safety
#else
            QRgb grayAsRgb = qRgb(value, value, value);
            m_paintedImage.setPixel(i, j, grayAsRgb);
#endif
            m_stim.at(i).at(j) = (value - 128.)/128;
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
            m_firingRate += m_stim.at(i).at(j) * spatial.at(i).at(j);
        }
    }
    m_firingRate /= resolutionHeight*resolutionWidth;
}

void RetinaEngine::stepEvent(double dt, bool parentEnabled)
{
    if(!parentEnabled || !isEnabled()  || !m_stim.size()){
        return;
    }

    std::random_device rd;
    std::mt19937 gen(rd());
    std::uniform_real_distribution<> dis(0,1);

    m_instantRate =  m_firingRate;
    emit instantRateChanged(m_instantRate);

    double shouldFire = (dis(gen) < m_sensitivity * m_firingRate * dt);
    if(shouldFire){
        fire();
    }
}

void RetinaEngine::resetPropertiesEvent()
{
    setSensitivity(1000.0);
}

void RetinaEngine::resetDynamicsEvent()
{
    m_firingRate = 0.0;
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

void RetinaEngine::setPlotKernel(bool plotKernel)
{
    if (m_plotKernel == plotKernel)
        return;

    m_plotKernel = plotKernel;
    emit plotKernelChanged(plotKernel);
}

void RetinaEngine::setSensitivity(double sensitivity)
{
    if (m_sensitivity == sensitivity)
        return;

    m_sensitivity = sensitivity;
    emit sensitivityChanged(sensitivity);
}


QImage RetinaEngine::paintedImage() const
{
    return m_paintedImage;
}




Kernel *RetinaEngine::kernel() const
{
    return m_kernel;
}

bool RetinaEngine::plotKernel() const
{
    return m_plotKernel;
}

double RetinaEngine::sensitivity() const
{
    return m_sensitivity;
}

double RetinaEngine::instantRate() const
{
    return m_instantRate;
}

VideoSurface *RetinaEngine::videoSurface() const
{
    return m_videoSurface;
}







