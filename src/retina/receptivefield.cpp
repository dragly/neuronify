#include "receptivefield.h"

#include <cmath>

const static long double pi = 3.141592653589793238462643383279502884L;

ReceptiveField::ReceptiveField()
{
}

void ReceptiveField::createOffLeftRF()
{

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            m_receptiveField.at(i).at(j)= 125;
        }
    }

    for(int i = 0; i < m_resolutionWidth/2; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            m_receptiveField.at(i).at(j) = -125;
        }
    }

}

void ReceptiveField::createOffRightRF()
{
    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            m_receptiveField.at(i).at(j)= -125;
        }
    }

    for(int i = 0; i < m_resolutionWidth/2; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            m_receptiveField.at(i).at(j) = 125;
        }
    }

}

void ReceptiveField::createOffTopRF()
{
    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            m_receptiveField.at(i).at(j)= 125;
        }
    }

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight/2; j++){
            m_receptiveField.at(i).at(j) = -125;
        }
    }
}

void ReceptiveField::createOffBottomRF()
{

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            m_receptiveField.at(i).at(j)= -125;
        }
    }

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight/2; j++){
            m_receptiveField.at(i).at(j) = 125;
        }
    }
}

void ReceptiveField::recreateRF()
{

    m_receptiveField.resize(m_resolutionWidth);
    for(int i = 0; i < m_resolutionWidth; i++){
        m_receptiveField.at(i).resize(m_resolutionHeight,0);
    }


    switch (m_receptiveFieldType) {
    case OffLeftRF:
        createOffLeftRF();
        break;
    case OffRightRF:
        createOffRightRF();
        break;
    case OffTopRF:
        createOffTopRF();
        break;
    case OffBottomRF:
        createOffBottomRF();
        break;
    case GaborRF:
        createGaborRF();
        break;
    default:
        createOffLeftRF();
        break;
    }



    m_image = QImage(m_resolutionWidth, m_resolutionHeight, QImage::Format_RGBA8888);

    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){

#ifdef Q_OS_ANDROID
            int gray = m_image.pixel(i,j);
#else
            int gray = rf().at(i).at(j) + 125;
            QRgb color = qRgb(gray, gray, gray);
            m_image.setPixel(i,j,color);
#endif
        }
    }

}

void ReceptiveField::createGaborRF()
{;
    for(int i = 0; i < m_resolutionWidth; i++){
        for(int j = 0; j < m_resolutionHeight; j++){
            m_receptiveField.at(i).at(j)= gaborFunction(i,j)*10000.;
//            qDebug() << m_receptiveField.at(i).at(j);
        }
    }

//    for(int i=0; i<m_resolutionHeight; i++)    //This loops on the rows.
//    {
//        for(int j=0; j<m_resolutionWidth; j++) //This loops on the columns
//        {
//            cout << setprecision(3)<< fixed << m_receptiveField[i][j]  << "  ";
//        }
//        cout << endl;
//    }

}



double ReceptiveField::gaborFunction(int idx, int idy)
{
    double sigmaX = 5.;
    double sigmaY = 5.;
    double k = 0.5;
    double phi = 0.0;
    double theta = 0.0;

    double x =  (idx - m_resolutionHeight/2) * cos(theta) + (idy-m_resolutionWidth/2) * sin(theta);
    double y = -(idx - m_resolutionHeight/2) * sin(theta) + (idy-m_resolutionWidth/2) * cos(theta);

    double prefactor = 1.0/(2.* pi * sigmaX * sigmaY);
    double expFactor = exp(-x*x/(2.* sigmaX * sigmaX) - y*y/(2. * sigmaY * sigmaY));
    double cosFactor = cos(k * x - phi);
    return prefactor * expFactor * cosFactor;
}


double ReceptiveField::temporalRF(const double tau)
{
    double alpha = 1.;
    return alpha*exp(-alpha*tau)*(pow(alpha*tau, 5)/120. - pow(alpha*tau, 7)/5040.);
}


int ReceptiveField::resolutionHeight() const
{
    return m_resolutionHeight;
}

int ReceptiveField::resolutionWidth() const
{
    return m_resolutionWidth;
}

vector<vector<double> > ReceptiveField::rf()
{
    if(m_receptiveField.empty()){
        recreateRF();
    }
    return m_receptiveField;
}

ReceptiveField::ReceptiveFieldTypes ReceptiveField::receptiveFieldType() const
{
    return m_receptiveFieldType;
}

QImage ReceptiveField::image() const
{
    return m_image;
}

void ReceptiveField::setRreceptiveFieldType(ReceptiveField::ReceptiveFieldTypes rfType)
{
    if (m_receptiveFieldType == rfType)
        return;

    m_receptiveFieldType = rfType;
    recreateRF();

    emit receptiveFieldTypeChanged(rfType);
}

void ReceptiveField::setResolutionHeight(int resolutionHeight)
{
    if (m_resolutionHeight == resolutionHeight)
        return;

    m_resolutionHeight = resolutionHeight;
    recreateRF();
    emit resolutionHeightChanged(resolutionHeight);
}

void ReceptiveField::setResolutionWidth(int resolutionWidth)
{
    if (m_resolutionWidth == resolutionWidth)
        return;

    m_resolutionWidth = resolutionWidth;
    recreateRF();
    emit resolutionWidthChanged(resolutionWidth);
}

void ReceptiveField::setImage(QImage image)
{
    if (m_image == image)
        return;

    m_image = image;
    emit imageChanged(image);
}



