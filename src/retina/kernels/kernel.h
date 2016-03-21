#ifndef KERNEL_H
#define KERNEL_H

#include <QQuickItem>
#include <vector>
#include <iostream>
#include <QImage>
#include<iomanip>
#include <limits>

using namespace std;

class Kernel : public QQuickItem
{
    Q_OBJECT
    Q_PROPERTY(int resolutionHeight READ resolutionHeight WRITE setResolutionHeight NOTIFY resolutionHeightChanged)
    Q_PROPERTY(int resolutionWidth READ resolutionWidth WRITE setResolutionWidth NOTIFY resolutionWidthChanged)
    Q_PROPERTY(QImage spatialImage READ spatialImage WRITE setSpatialImage NOTIFY imageChanged)
    Q_PROPERTY(spatialTypes spatialType READ spatialType WRITE setSpatialType NOTIFY spatialTypeChanged)
    Q_ENUMS(spatialTypes)

public:
    Kernel();

public:
    enum spatialTypes{
        OffLeftRF,
        OffRightRF,
        OffTopRF,
        OffBottomRF,
        GaborRF
    };

    int resolutionHeight() const;
    int resolutionWidth() const;
    void recreate();

    //Kernel types:
    void createOffLeft();
    void createOffRight();
    void createOffTop();
    void createOffBottom();
    void createGabor();
    double gaborFunction(int x, int y);


    vector<vector<double> > spatial();
    spatialTypes spatialType() const;


    QImage spatialImage() const;

public slots:
    void setSpatialType(spatialTypes spatialType);
    void setResolutionHeight(int resolutionHeight);
    void setResolutionWidth(int resolutionWidth);

    void setSpatialImage(QImage spatialImage);

signals:
    void resolutionHeightChanged(int resolutionHeight);
    void resolutionWidthChanged(int resolutionWidth);
    void spatialTypeChanged(spatialTypes spatialType);

    void imageChanged(QImage spatialImage);

private:
    int m_resolutionHeight = 10;
    int m_resolutionWidth = 10;
    vector< vector <double>> m_spatial;
    spatialTypes m_spatialType = OffLeftRF;
    QImage m_spatialImage;
};


#endif // KERNEL_H
