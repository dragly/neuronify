#ifndef RETINAENGINE_H
#define RETINAENGINE_H

#include <iostream>
#include <vector>
#include <math.h>
#include <QVideoRendererControl>

#include "videosurface.h"
#include "nodeengine.h"

using namespace std;

class RetinaEngine : public NodeEngine
{
    Q_OBJECT
    Q_PROPERTY(VideoSurface *  videoSurface READ videoSurface WRITE setVideoSurface NOTIFY videoSurfaceChanged)


public:
    RetinaEngine();
    ~RetinaEngine();

    void calculateFiringRate();
    double temporalRF(const double tau);
    double gaborField(int x, int y);

    VideoSurface * videoSurface() const;
    QImage image() const;

public slots:
    void receivedImage();
    void setVideoSurface(VideoSurface * videoSurface);

signals:
    void videoSurfaceChanged(VideoSurface * videoSurface);

protected:
    virtual void stepEvent(double dt);

private:
    VideoSurface * m_videoSurface = nullptr;
    QImage m_image;

    vector< vector <double>> m_stim;
    vector< vector <double>> m_recField;

    int m_nPixelsX;
    int m_nPixelsY;
    double m_firingRate = 0.0;

    void makeReceptiveField();
};


#endif // RETINAENGINE_H
