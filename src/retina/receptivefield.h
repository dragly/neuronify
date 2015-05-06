#ifndef RECEPTIVEFIELD_H
#define RECEPTIVEFIELD_H

#include <QQuickItem>
#include <vector>
#include <iostream>
 #include <QImage>

using namespace std;

class ReceptiveField : public QQuickItem
{
    Q_OBJECT
    Q_PROPERTY(int nPixelsX READ nPixelsX WRITE setNPixelsX NOTIFY nPixelsXChanged)
    Q_PROPERTY(int nPixelsY READ nPixelsY WRITE setNPixelsY NOTIFY nPixelsYChanged)
    Q_PROPERTY(QImage image READ image WRITE setImage NOTIFY imageChanged)
    Q_PROPERTY(ReceptiveFieldTypes receptiveFieldType READ receptiveFieldType WRITE setRreceptiveFieldType NOTIFY receptiveFieldTypeChanged)
    Q_ENUMS(ReceptiveFieldTypes)

public:
    ReceptiveField();

public:
    enum ReceptiveFieldTypes{
        OffLeftRF,
        OffRightRF,
        OffTopRF,
        OffBottomRF
    };

    int nPixelsX() const;
    int nPixelsY() const;
    void recreateRF();

    //Receptive Field types:
    void createOffLeftRF();
    void createOffRightRF();
    void createOffTopRF();
    void createOffBottomRF();
    double temporalRF(const double tau);
    double gaborField(int x, int y);


    vector<vector<double> > rf();
    ReceptiveFieldTypes receptiveFieldType() const;


    QImage image() const
    {
        return m_image;
    }

public slots:
    void setRreceptiveFieldType(ReceptiveFieldTypes receptiveFieldType);
    void setNPixelsX(int nPixelsX);
    void setNPixelsY(int nPixelsY);

    void setImage(QImage image)
    {
        if (m_image == image)
            return;

        m_image = image;
        emit imageChanged(image);
    }

signals:
    void nPixelsXChanged(int nPixelsX);
    void nPixelsYChanged(int nPixelsY);
    void receptiveFieldTypeChanged(ReceptiveFieldTypes receptiveFieldType);

    void imageChanged(QImage image);

private:
    int m_nPixelsX = 10;
    int m_nPixelsY = 10;
    vector< vector <double>> m_receptiveField;
    ReceptiveFieldTypes m_receptiveFieldType = OffLeftRF;
    QImage m_image;
};


#endif // RECEPTIVEFIELD_H
