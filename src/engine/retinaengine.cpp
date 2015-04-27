#include "retinaengine.h"

#include <QPainter>
#include <QVideoRendererControl>

RetinaEngine::RetinaEngine()
{
}

RetinaEngine::~RetinaEngine()
{
}

void RetinaEngine::receivedImage()
{

    if(!m_recField){
        return;
    }
    int nPixelsX= m_recField->nPixelsX();
    int nPixelsY = m_recField->nPixelsY();
    m_stim.resize(nPixelsX);
    for(int i = 0; i < nPixelsX; i++){
        m_stim.at(i).resize(nPixelsY,0.0);
    }

    m_image = m_videoSurface->image();
    m_image =  m_image.scaled(nPixelsX,nPixelsY);

    for(int i = 0; i < m_image.width(); i++){
        for(int j = 0; j < m_image.height(); j++){

            int gray = qGray(m_image.pixel(i,j));
            m_stim.at(i).at(j) = gray-126.;
            QRgb color = qRgb(gray, gray, gray);
            m_image.setPixel(i,j,color);
        }
    }

    calculateFiringRate();
    update();
}

void RetinaEngine::calculateFiringRate()
{
    if(!m_recField){
        return;
    }
    m_rf = m_recField->rf();
    int nPixelsX= m_recField->nPixelsX();
    int nPixelsY = m_recField->nPixelsY();

    for(int i = 0; i < nPixelsX; i++){
        for(int j = 0; j < nPixelsY; j++){
            //            qDebug() << m_stim.at(i).at(j)  << "    " << m_rf.at(i).at(j);
            m_firingRate += m_stim.at(i).at(j) *  m_rf.at(i).at(j) * 1./nPixelsX/nPixelsY;
        }
    }
}



void RetinaEngine::stepEvent(double dt)
{

    if(m_firingRate < 0){
        m_firingRate = 0;
    }

    std::random_device rd;
    std::mt19937 gen(rd());
    std::uniform_real_distribution<> dis(0,1);

    double shouldFire = (dis(gen) < m_firingRate*dt);
    if(shouldFire){
        fire();
        m_firingRate = 0;
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



void RetinaEngine::setRecField(ReceptiveField *recField)
{
    if (m_recField == recField)
        return;

//    if(m_recField){
//        disconnect(m_recField, &ReceptiveField::nPixelsXChanged, this, &RetinaEngine::setNPixelsX);
//        disconnect(m_recField, &ReceptiveField::nPixelsYChanged, this, &RetinaEngine::setNPixelsY);
//    }

    m_recField = recField;

//    if(m_recField){
//        connect(m_recField, &ReceptiveField::nPixelsXChanged, this, &RetinaEngine::setNPixelsX);
//        connect(m_recField, &ReceptiveField::nPixelsYChanged, this, &RetinaEngine::setNPixelsY);
//    }


    emit recFieldChanged(recField);
}


QImage RetinaEngine::image() const
{
    return m_image;
}

ReceptiveField *RetinaEngine::recField() const
{
    return m_recField;
}

VideoSurface *RetinaEngine::videoSurface() const
{
    return m_videoSurface;
}








