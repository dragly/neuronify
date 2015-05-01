#include "retinaengine.h"

#include <QPainter>
#include <QVideoRendererControl>
#include <random>

RetinaEngine::RetinaEngine()
{
}

RetinaEngine::~RetinaEngine()
{
}

void RetinaEngine::receivedImage()
{
    qDebug() << "Received image";
    if(!m_receptiveField){
        return;
    }
    int nPixelsX = m_receptiveField->nPixelsX();
    int nPixelsY = m_receptiveField->nPixelsY();

    m_stim.resize(nPixelsX);
    for(int i = 0; i < nPixelsX; i++){
        m_stim.at(i).resize(nPixelsY,0.0);
    }

    m_image = m_videoSurface->image();
    m_image =  m_image.scaled(nPixelsY,nPixelsX);


    for(int i = 0; i < m_image.width(); i++){
        for(int j = 0; j < m_image.height(); j++){

            int gray = qGray(m_image.pixel(i,j));
            m_stim.at(j).at(i) = gray-126.;
            QRgb color = qRgb(gray, gray, gray);
            m_image.setPixel(i,j,color);
        }
    }

    calculateFiringRate();
    update();
}

void RetinaEngine::calculateFiringRate()
{
    if(!m_receptiveField){
        return;
    }
    m_receptiveFieldShape = m_receptiveField->rf();
    int nPixelsX= m_receptiveField->nPixelsX();
    int nPixelsY = m_receptiveField->nPixelsY();

    m_firingRate = 0.0;
    for(int i = 0; i < nPixelsX; i++){
        for(int j = 0; j < nPixelsY; j++){
            //            qDebug() << m_stim.at(i).at(j)  << "    " << m_rf.at(i).at(j);
            double factor = 1.0;
#ifdef Q_OS_ANDROID
            factor = 1.0;
#endif
            m_firingRate += factor * m_stim.at(i).at(j) *  m_receptiveFieldShape.at(i).at(j) * 1./nPixelsX/nPixelsY;
        }
    }
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



void RetinaEngine::setReceptiveField(ReceptiveField *recField)
{
    if (m_receptiveField == recField)
        return;

    m_receptiveField = recField;

    emit receptiveFieldChanged(recField);
}


QImage RetinaEngine::image() const
{
    return m_image;
}

ReceptiveField *RetinaEngine::receptiveField() const
{
    return m_receptiveField;
}

VideoSurface *RetinaEngine::videoSurface() const
{
    return m_videoSurface;
}








