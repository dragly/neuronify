#ifndef RETINAENGINE_H
#define RETINAENGINE_H

#include <iostream>
#include <vector>
#include <math.h>
#include <QVideoRendererControl>

#include "videosurface.h"
#include "nodeengine.h"
#include "receptivefield.h"

using namespace std;

class RetinaEngine : public NodeEngine
{
    Q_OBJECT
    Q_PROPERTY(VideoSurface *  videoSurface READ videoSurface WRITE setVideoSurface NOTIFY videoSurfaceChanged)
    Q_PROPERTY(ReceptiveField * recField READ recField WRITE setRecField NOTIFY recFieldChanged)


public:
    RetinaEngine();
    ~RetinaEngine();

    void calculateFiringRate();

    VideoSurface * videoSurface() const;
    QImage image() const;
    ReceptiveField * recField() const;

public slots:
    void receivedImage();
    void setVideoSurface(VideoSurface * videoSurface);
    void setRecField(ReceptiveField * recField);

signals:
    void videoSurfaceChanged(VideoSurface * videoSurface);
    void recFieldChanged(ReceptiveField * recField);

protected:
    virtual void stepEvent(double dt);

private:
    VideoSurface * m_videoSurface = nullptr;
    ReceptiveField * m_recField = nullptr;
    QImage m_image;

    double m_firingRate = 0.0;

    vector< vector <double>> m_stim;
    vector< vector <double>> m_rf;

};


#endif // RETINAENGINE_H
