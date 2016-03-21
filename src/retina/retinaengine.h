#ifndef RETINAENGINE_H
#define RETINAENGINE_H

#include <iostream>
#include <vector>
#include <math.h>
#include <QVideoRendererControl>

#include "videosurface.h"
#include "../core/nodeengine.h"
#include "kernels/kernel.h"

using namespace std;

class RetinaEngine : public NodeEngine
{
    Q_OBJECT
    Q_PROPERTY(VideoSurface *  videoSurface READ videoSurface WRITE setVideoSurface NOTIFY videoSurfaceChanged)
    Q_PROPERTY(Kernel * kernel READ kernel WRITE setKernel NOTIFY kernelChanged)
    Q_PROPERTY(bool plotKernel READ plotKernel WRITE setPlotKernel NOTIFY plotKernelChanged)

public:
    RetinaEngine();
    ~RetinaEngine();

    void calculateFiringRate();


    VideoSurface * videoSurface() const;
    QImage paintedImage() const;
    Kernel * kernel() const;

    bool plotKernel() const
    {
        return m_plotKernel;
    }

public slots:
    void receivedImage();
    void setVideoSurface(VideoSurface * videoSurface);
    void setKernel(Kernel * kernel);

    void setPlotKernel(bool plotKernel)
    {
        if (m_plotKernel == plotKernel)
            return;

        m_plotKernel = plotKernel;
        emit plotKernelChanged(plotKernel);
    }

signals:
    void videoSurfaceChanged(VideoSurface * videoSurface);
    void kernelChanged(Kernel * kernel);
    void plotKernelChanged(bool plotKernel);

protected:
    virtual void stepEvent(double dt);

private:
    VideoSurface * m_videoSurface = nullptr;
    Kernel * m_kernel = nullptr;
    QImage m_paintedImage;


    double m_firingRate = 0.0;

    vector< vector <double>> m_stim;

    bool m_plotKernel;
};


#endif // RETINAENGINE_H
