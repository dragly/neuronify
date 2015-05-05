#ifndef RECEPTIVEFIELD_H
#define RECEPTIVEFIELD_H

#include <QQuickItem>
#include <vector>
#include <iostream>
#include<iomanip>

using namespace std;

class ReceptiveField : public QQuickItem
{
    Q_OBJECT
    Q_PROPERTY(int nPixelsX READ nPixelsX WRITE setNPixelsX NOTIFY nPixelsXChanged)
    Q_PROPERTY(int nPixelsY READ nPixelsY WRITE setNPixelsY NOTIFY nPixelsYChanged)
    Q_PROPERTY(ReceptiveFieldTypes receptiveFieldType READ receptiveFieldType WRITE setRreceptiveFieldType NOTIFY receptiveFieldTypeChanged)
    Q_ENUMS(ReceptiveFieldTypes)

public:
    ReceptiveField();

public:
    enum ReceptiveFieldTypes{
        OffLeftRF,
        OffRightRF,
        OffTopRF,
        OffBottomRF,
        GaborRF
    };

    int nPixelsX() const;
    int nPixelsY() const;
    void recreateRF();

    //Receptive Field types:
    void createOffLeftRF();
    void createOffRightRF();
    void createOffTopRF();
    void createOffBottomRF();
    void createGaborRF();
    double temporalRF(const double tau);
    double gaborFunction(int x, int y);


    vector<vector<double> > rf();
    ReceptiveFieldTypes receptiveFieldType() const;


public slots:
    void setRreceptiveFieldType(ReceptiveFieldTypes receptiveFieldType);
    void setNPixelsX(int nPixelsX);
    void setNPixelsY(int nPixelsY);

signals:
    void nPixelsXChanged(int nPixelsX);
    void nPixelsYChanged(int nPixelsY);
    void receptiveFieldTypeChanged(ReceptiveFieldTypes receptiveFieldType);

private:
    int m_nPixelsX = 10;
    int m_nPixelsY = 10;
    vector< vector <double>> m_receptiveField;
    ReceptiveFieldTypes m_receptiveFieldType = OffLeftRF;
};


#endif // RECEPTIVEFIELD_H
