#include "receptivefield.h"

#include <cmath>

const static long double pi = 3.141592653589793238462643383279502884L;

ReceptiveField::ReceptiveField()
{
}

void ReceptiveField::createOffLeftRF()
{
    for(int i = 0; i < m_nPixelsX; i++){
        for(int j = 0; j < m_nPixelsY; j++){
            m_receptiveField.at(i).at(j)= 1;
        }
    }

    for(int i = 0; i < m_nPixelsX; i++){
        for(int j = 0; j < m_nPixelsY/2; j++){
            m_receptiveField.at(i).at(j) = -1;
        }
    }
}

void ReceptiveField::createOffRightRF()
{
    for(int i = 0; i < m_nPixelsX; i++){
        for(int j = 0; j < m_nPixelsY; j++){
            m_receptiveField.at(i).at(j)= -1;
        }
    }

    for(int i = 0; i < m_nPixelsX; i++){
        for(int j = 0; j < m_nPixelsY/2; j++){
            m_receptiveField.at(i).at(j) = 1;
        }
    }
}

void ReceptiveField::createOffTopRF()
{
    for(int i = 0; i < m_nPixelsX; i++){
        for(int j = 0; j < m_nPixelsY; j++){
            m_receptiveField.at(i).at(j)= 1;
        }
    }

    for(int i = 0; i < m_nPixelsX/2; i++){
        for(int j = 0; j < m_nPixelsY; j++){
            m_receptiveField.at(i).at(j) = -1;
        }
    }
}

void ReceptiveField::createOffBottomRF()
{;
    for(int i = 0; i < m_nPixelsX; i++){
        for(int j = 0; j < m_nPixelsY; j++){
            m_receptiveField.at(i).at(j)= -1;
        }
    }

    for(int i = 0; i < m_nPixelsX/2; i++){
        for(int j = 0; j < m_nPixelsY; j++){
            m_receptiveField.at(i).at(j) = 1;
        }
    }
}

void ReceptiveField::recreateRF()
{

    m_receptiveField.resize(m_nPixelsX);
    for(int i = 0; i < m_nPixelsX; i++){
        m_receptiveField.at(i).resize(m_nPixelsY,0);
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
    m_image = QImage(m_nPixelsY, m_nPixelsX, QImage::Format_RGBA8888);
    for(int i = 0; i < m_image.width(); i++){
        for(int j = 0; j < m_image.height(); j++){

#ifdef Q_OS_ANDROID
            int gray = m_image.pixel(i,j);
#else
            int gray = rf().at(m_nPixelsX-1-j).at(m_nPixelsY-1-i);
            QRgb color = qRgb(gray, gray, gray);
            m_image.setPixel(i,j,color);
#endif
        }
    }

}

void ReceptiveField::createGaborRF()
{;
    for(int i = 0; i < m_nPixelsX; i++){
        for(int j = 0; j < m_nPixelsY; j++){
            m_receptiveField.at(i).at(j)= gaborFunction(i,j)*1000.;
        }
    }

//    for(int i=0; i<m_nPixelsX; i++)    //This loops on the rows.
//    {
//        for(int j=0; j<m_nPixelsY; j++) //This loops on the columns
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

    double x =  (idx - m_nPixelsX/2) * cos(theta) + (idy-m_nPixelsY/2) * sin(theta);
    double y = -(idx - m_nPixelsX/2) * sin(theta) + (idy-m_nPixelsY/2) * cos(theta);

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


int ReceptiveField::nPixelsX() const
{
    return m_nPixelsX;
}

int ReceptiveField::nPixelsY() const
{
    return m_nPixelsY;
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

void ReceptiveField::setRreceptiveFieldType(ReceptiveField::ReceptiveFieldTypes rfType)
{
    if (m_receptiveFieldType == rfType)
        return;

    m_receptiveFieldType = rfType;
    recreateRF();

    emit receptiveFieldTypeChanged(rfType);
}

void ReceptiveField::setNPixelsX(int nPixelsX)
{
    if (m_nPixelsX == nPixelsX)
        return;

    m_nPixelsX = nPixelsX;
    recreateRF();
    emit nPixelsXChanged(nPixelsX);
}

void ReceptiveField::setNPixelsY(int nPixelsY)
{
    if (m_nPixelsY == nPixelsY)
        return;

    m_nPixelsY = nPixelsY;
    recreateRF();
    emit nPixelsYChanged(nPixelsY);
}



