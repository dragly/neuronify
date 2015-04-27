#include "receptivefield.h"

ReceptiveField::ReceptiveField()
{
    //    setReceptiveField();
}

//void ReceptiveField::setReceptiveField()
//{
//    m_rf.resize(m_nPixelsX);
//    for(int i = 0; i < m_nPixelsX; i++){
//        m_rf.at(i).resize(m_nPixelsY,1);
//    }

//    for(int i = 0; i < m_nPixelsX; i++){
//        for(int j = 0; j < m_nPixelsY/2; j++){
//            m_rf.at(i).at(j) = -1;
//        }
//    }

//    //    for(int i=0; i<m_nPixelsX; i++)    //This loops on the rows.
//    //    {
//    //        for(int j=0; j<m_nPixelsY; j++) //This loops on the columns
//    //        {
//    //            cout << m_rf[i][j]  << "  ";
//    //        }
//    //        cout << endl;
//    //    }


//}


void ReceptiveField::createOffLeftRF()
{
    for(int i = 0; i < m_nPixelsX; i++){
        for(int j = 0; j <m_nPixelsY; j++){
            m_rf.at(i).at(j)= 1;
        }
    }

    for(int i = 0; i < m_nPixelsX; i++){
        for(int j = 0; j < m_nPixelsY/2; j++){
            m_rf.at(i).at(j) = -1;
        }
    }

}

void ReceptiveField::createOffRightRF()
{
    for(int i = 0; i < m_nPixelsX; i++){
        for(int j = 0; j <m_nPixelsY; j++){
            m_rf.at(i).at(j)= -1;
        }
    }

    for(int i = 0; i < m_nPixelsX; i++){
        for(int j = 0; j < m_nPixelsY/2; j++){
            m_rf.at(i).at(j) = 1;
        }
    }

}

void ReceptiveField::recreateRF()
{

    m_rf.resize(m_nPixelsX);
    for(int i = 0; i < m_nPixelsX; i++){
        m_rf.at(i).resize(m_nPixelsY,0);
    }

    switch (m_rfType) {
    case OffLeftRF:
        createOffLeftRF();
        break;
    case OffRightRF:
        createOffRightRF();
        break;
    default:
        createOffLeftRF();
        break;
    }
}



double ReceptiveField::temporalRF(const double tau)
{
    double alpha = 1.;
    return alpha*exp(-alpha*tau)*(pow(alpha*tau, 5)/120. - pow(alpha*tau, 7)/5040.);
}


double ReceptiveField::gaborField(int idx, int idy)
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



int ReceptiveField::nPixelsX() const
{
    return m_nPixelsX;
}

int ReceptiveField::nPixelsY() const
{
    return m_nPixelsY;
}

vector<vector<double> > ReceptiveField::rf() const
{
    return m_rf;
}

ReceptiveField::ReceptiveFieldTypes ReceptiveField::rfType() const
{
    return m_rfType;
}

void ReceptiveField::setRfType(ReceptiveField::ReceptiveFieldTypes rfType)
{
    if (m_rfType == rfType)
        return;

    m_rfType = rfType;
    recreateRF();

    emit rfTypeChanged(rfType);
}

void ReceptiveField::setNPixelsX(int nPixelsX)
{
    if (m_nPixelsX == nPixelsX)
        return;

    m_nPixelsX = nPixelsX;
    emit nPixelsXChanged(nPixelsX);
    recreateRF();
}

void ReceptiveField::setNPixelsY(int nPixelsY)
{
    if (m_nPixelsY == nPixelsY)
        return;

    m_nPixelsY = nPixelsY;
    emit nPixelsYChanged(nPixelsY);
    recreateRF();
}



