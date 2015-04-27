#ifndef RECEPTIVEFIELD_H
#define RECEPTIVEFIELD_H

#include <QQuickItem>
#include <vector>
#include <iostream>

using namespace std;

class ReceptiveField : public QQuickItem
{
    Q_OBJECT
    Q_PROPERTY(int nPixelsX READ nPixelsX WRITE setNPixelsX NOTIFY nPixelsXChanged)
    Q_PROPERTY(int nPixelsY READ nPixelsY WRITE setNPixelsY NOTIFY nPixelsYChanged)
    Q_PROPERTY(ReceptiveFieldTypes rfType READ rfType WRITE setRfType NOTIFY rfTypeChanged)
    Q_ENUMS(ReceptiveFieldTypes)

public:
    ReceptiveField();

public:
    enum ReceptiveFieldTypes{
        OffLeftRF,
        OffRightRF
    };

    double temporalRF(const double tau);
    double gaborField(int x, int y);
//    void setReceptiveField();
    void createOffLeftRF();
    void createOffRightRF();
    void recreateRF();

    int nPixelsX() const;
    int nPixelsY() const;

    vector<vector<double> > rf() const;
    ReceptiveFieldTypes rfType() const;




public slots:

    void setRfType(ReceptiveFieldTypes rfType);
    void setNPixelsX(int nPixelsX);
    void setNPixelsY(int nPixelsY);

signals:
    void nPixelsXChanged(int nPixelsX);
    void nPixelsYChanged(int nPixelsY);
    void rfTypeChanged(ReceptiveFieldTypes rfType);

private:
    int m_nPixelsX = 300;
    int m_nPixelsY = 300;
    vector< vector <double>> m_rf;

    ReceptiveFieldTypes m_rfType;
};


#endif // RECEPTIVEFIELD_H
