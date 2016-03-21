#include "kernel.h"

#include <cmath>

/*!
 * \class Kernel
 * \inmodule Neuronify
 * \ingroup neuronify-sensors
 * \brief Creates different kernel types.
 */

const static long double pi = 3.141592653589793238462643383279502884L;

Kernel::Kernel()
{
}

void Kernel::createOffLeft()
{

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            m_spatial.at(i).at(j) = 127;
        }
    }

    for(int i = 0; i < m_resolutionWidth/2; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            m_spatial.at(i).at(j) = -128;
        }
    }

}

void Kernel::createOffRight()
{
    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            m_spatial.at(i).at(j)= -128;
        }
    }

    for(int i = 0; i < m_resolutionWidth/2; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            m_spatial.at(i).at(j) = 127;
        }
    }

}

void Kernel::createOffTop()
{
    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            m_spatial.at(i).at(j)= 127;
        }
    }

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight/2; j++){
            m_spatial.at(i).at(j) = -128;
        }
    }
}

void Kernel::createOffBottom()
{

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            m_spatial.at(i).at(j)= -128;
        }
    }

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight/2; j++){
            m_spatial.at(i).at(j) = 127;
        }
    }
}

void Kernel::recreate()
{

    m_spatial.resize(m_resolutionWidth);
    for(int i = 0; i < m_resolutionWidth; i++){
        m_spatial.at(i).resize(m_resolutionHeight,0);
    }


    switch (m_spatialType) {
    case OffLeftRF:
        createOffLeft();
        break;
    case OffRightRF:
        createOffRight();
        break;
    case OffTopRF:
        createOffTop();
        break;
    case OffBottomRF:
        createOffBottom();
        break;
    case GaborRF:
        createGabor();
        break;
    default:
        createOffLeft();
        break;
    }



    m_spatialImage = QImage(m_resolutionWidth, m_resolutionHeight, QImage::Format_RGBA8888);

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){

#ifdef Q_OS_ANDROID
            int gray = m_spatialImage.pixel(i,j);
#else
            int gray = spatial().at(i).at(j) + 128;
            QRgb color = qRgb(gray, gray, gray);
            m_spatialImage.setPixel(i,j,color);
#endif
        }
    }

}

void Kernel::createGabor()
{;
    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            m_spatial.at(i).at(j)= gaborFunction(i,j)*20000.;
//            qDebug() << m_kernel.at(i).at(j);
        }
    }

//    for(int i=0; i<m_resolutionHeight; i++)    //This loops on the rows.
//    {
//        for(int j=0; j<m_resolutionWidth; j++) //This loops on the columns
//        {
//            cout << setprecision(3)<< fixed << m_kernel[i][j]  << "  ";
//        }
//        cout << endl;
//    }

}



double Kernel::gaborFunction(int idx, int idy)
{
    double sigmaX = 5.;
    double sigmaY = 5.;
    double k = 0.5;
    double phi = 0.0;
    double theta = 0.0;

    double x =  (idx - m_resolutionHeight/2) * cos(theta)
            + (idy-m_resolutionWidth/2) * sin(theta);
    double y = -(idx - m_resolutionHeight/2) * sin(theta)
            + (idy-m_resolutionWidth/2) * cos(theta);

    double prefactor = 1.0/(2.* pi * sigmaX * sigmaY);
    double expFactor = exp(-x*x/(2.* sigmaX * sigmaX) - y*y/(2. * sigmaY * sigmaY));
    double cosFactor = cos(k * x - phi);
    return prefactor * expFactor * cosFactor;
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

Kernel::spatialTypes Kernel::spatialType() const
{
    return m_spatialType;
}

QImage Kernel::spatialImage() const
{
    return m_spatialImage;
}

void Kernel::setSpatialType(Kernel::spatialTypes spatialType)
{
    if (m_spatialType == spatialType)
        return;

    m_spatialType = spatialType;
    recreate();

    emit spatialTypeChanged(spatialType);
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



