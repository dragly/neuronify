#include "retinaengine.h"

#include <QPainter>
#include <QVideoRendererControl>

RetinaEngine::RetinaEngine():
    m_nPixelsX(300),
    m_nPixelsY(300)
{
    makeReceptiveField();
    connect(&m_videoSurface, &VideoSurface::gotImage, this, &RetinaEngine::receivedImage);
}

RetinaEngine::~RetinaEngine()
{

}

void RetinaEngine::receivedImage()
{
    m_image = m_videoSurface.image();
    m_image =  m_image.scaled(m_nPixelsX,m_nPixelsY);

    for(int i = 0; i < m_image.width(); i++){
        for(int j = 0; j < m_image.height(); j++){

            int gray = qGray(m_image.pixel(j,i));
            m_stim.at(i).at(j) = gray-126.;
            QRgb color = qRgb(gray, gray, gray);
            m_image.setPixel(j,i,color);
        }
    }

    calculateFiringRate();
    update();
}

void RetinaEngine::setCamera(QObject* camera)
{
    if (m_camera == camera)
        return;

    m_camera = camera;

    QCamera *cameraObject = qvariant_cast<QCamera *>(camera->property("mediaObject"));

    if(cameraObject) {
        m_rendererControl = cameraObject->service()->requestControl<QVideoRendererControl *>();
        if(m_rendererControl) {
            m_rendererControl->setSurface(&m_videoSurface);
        }
    }

    emit cameraChanged(camera);
}

QImage RetinaEngine::image() const
{
    return m_image;
}

QObject* RetinaEngine::camera() const
{
    return m_camera;
}


void RetinaEngine::makeReceptiveField()
{
    m_recField.resize(m_nPixelsX);
    m_stim.resize(m_nPixelsX);
    for(int i = 0; i < m_nPixelsX; i++){
        m_recField.at(i).resize(m_nPixelsY,1);
        m_stim.at(i).resize(m_nPixelsY,0.0);
    }


    for(int i = 0; i < m_nPixelsX; i++){
        for(int j = 0; j < m_nPixelsY/2; j++){
            m_recField.at(i).at(j) = -1;
        }
    }

    //    for(int i=0; i<m_nPixelsX; i++)    //This loops on the rows.
    //    {
    //        for(int j=0; j<m_nPixelsY; j++) //This loops on the columns
    //        {
    //            cout << m_recField[i][j]  << "  ";
    //        }
    //        cout << endl;
    //    }

}

void RetinaEngine::calculateFiringRate()
{

    for(int i = 0; i < m_nPixelsX; i++){
        for(int j = 0; j < m_nPixelsY; j++){
            //            qDebug() << m_stim.at(i).at(j)  << "    " << m_recField.at(i).at(j);
            m_firingRate += m_stim.at(i).at(j) *  m_recField.at(i).at(j) * 1./m_nPixelsX/m_nPixelsY;
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


double RetinaEngine::temporalRF(const double tau)
{
    double alpha = 1.;
    return alpha*exp(-alpha*tau)*(pow(alpha*tau, 5)/120. - pow(alpha*tau, 7)/5040.);
}


double RetinaEngine::gaborField(int idx, int idy)
{
    double sigmaX = 400.;
    double sigmaY = 400.;
    double k = 1;
    double phi = 0.0;
    double theta = 0.0;

    double x =  idx * cos(theta) + idy * sin(theta);
    double y = -idx * sin(theta) + idy * cos(theta);

    double prefactor = 1.0/(2.* M_PI * sigmaX * sigmaY);
    double expFactor = exp(-x*x/(2.* sigmaX * sigmaX) - y*y/(2. * sigmaY * sigmaY));
    double cosFactor = cos(k * x - phi);
    return prefactor * expFactor * cosFactor;
}






