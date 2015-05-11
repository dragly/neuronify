#ifndef RECEPTIVEFIELD_H
#define RECEPTIVEFIELD_H

#include <QQuickItem>
#include <vector>
#include <iostream>
#include <QImage>
#include<iomanip>
#include <limits>

using namespace std;

class ReceptiveField : public QQuickItem
{
    Q_OBJECT
    Q_PROPERTY(int resolutionHeight READ resolutionHeight WRITE setResolutionHeight NOTIFY resolutionHeightChanged)
    Q_PROPERTY(int resolutionWidth READ resolutionWidth WRITE setResolutionWidth NOTIFY resolutionWidthChanged)
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
        OffBottomRF,
        GaborRF
    };

    int resolutionHeight() const;
    int resolutionWidth() const;
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


    QImage image() const;

public slots:
    void setRreceptiveFieldType(ReceptiveFieldTypes receptiveFieldType);
    void setResolutionHeight(int resolutionHeight);
    void setResolutionWidth(int resolutionWidth);

    void setImage(QImage image);

signals:
    void resolutionHeightChanged(int resolutionHeight);
    void resolutionWidthChanged(int resolutionWidth);
    void receptiveFieldTypeChanged(ReceptiveFieldTypes receptiveFieldType);

    void imageChanged(QImage image);

private:
    int m_resolutionHeight = 10;
    int m_resolutionWidth = 10;
    vector< vector <double>> m_receptiveField;
    ReceptiveFieldTypes m_receptiveFieldType = OffLeftRF;
    QImage m_image;
};


#endif // RECEPTIVEFIELD_H
