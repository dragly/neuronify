#ifndef RETINAENGINE_H
#define RETINAENGINE_H

#include <QCamera>
#include <iostream>
#include <vector>
#include <math.h>

#include "videosurface.h"
#include "nodeengine.h"

using namespace std;

class RetinaEngine : public NodeEngine
{
    Q_OBJECT

public:
    RetinaEngine();
    ~RetinaEngine();

    void startCamera();
    void calculateFiringRate();
    double temporalRF(const double tau);
    double gaborField(int x, int y);

    QImage image() const;

public slots:
    void receivedImage();

protected:
    virtual void stepEvent(double dt);

private:
    QCamera* m_camera;
    VideoSurface m_videoSurface;
    QImage m_image;

    vector< vector <double>> m_stim;
    vector< vector <double>> m_recField;

    int m_nPixelsX;
    int m_nPixelsY;
    double m_firingRate = 0.0;

    void makeReceptiveField();

};


#endif // RETINAENGINE_H
