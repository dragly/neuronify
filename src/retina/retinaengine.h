#ifndef RETINAENGINE_H
#define RETINAENGINE_H

#include <iostream>
#include <vector>
#include <math.h>
#include <QVideoRendererControl>

#include "videosurface.h"
#include "../core/nodeengine.h"
#include "receptivefield.h"

using namespace std;

class RetinaEngine : public NodeEngine
{
    Q_OBJECT
    Q_PROPERTY(VideoSurface *  videoSurface READ videoSurface WRITE setVideoSurface NOTIFY videoSurfaceChanged)
    Q_PROPERTY(ReceptiveField * receptiveField READ receptiveField WRITE setReceptiveField NOTIFY receptiveFieldChanged)
    Q_PROPERTY(bool plotReceptiveField READ plotReceptiveField WRITE setPlotReceptiveField NOTIFY plotReceptiveFieldChanged)

public:
    RetinaEngine();
    ~RetinaEngine();

    void calculateFiringRate();


    VideoSurface * videoSurface() const;
    QImage paintedImage() const;
    ReceptiveField * receptiveField() const;

    bool plotReceptiveField() const
    {
        return m_plotReceptiveField;
    }

public slots:
    void receivedImage();
    void setVideoSurface(VideoSurface * videoSurface);
    void setReceptiveField(ReceptiveField * receptiveField);

    void setPlotReceptiveField(bool plotReceptiveField)
    {
        if (m_plotReceptiveField == plotReceptiveField)
            return;

        m_plotReceptiveField = plotReceptiveField;
        emit plotReceptiveFieldChanged(plotReceptiveField);
    }

signals:
    void videoSurfaceChanged(VideoSurface * videoSurface);
    void receptiveFieldChanged(ReceptiveField * receptiveField);

    void plotReceptiveFieldChanged(bool plotReceptiveField);

protected:
    virtual void stepEvent(double dt);

private:
    VideoSurface * m_videoSurface = nullptr;
    ReceptiveField * m_receptiveField = nullptr;
    QImage m_paintedImage;


    double m_firingRate = 0.0;

    vector< vector <double>> m_stim;

    bool m_plotReceptiveField;
};


#endif // RETINAENGINE_H
