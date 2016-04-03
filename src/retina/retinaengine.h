#ifndef RETINAENGINE_H
#define RETINAENGINE_H

#include <iostream>
#include <vector>
#include <math.h>
#include <QVideoRendererControl>

#include "videosurface.h"
#include "../core/nodeengine.h"
#include "kernel.h"

using namespace std;

class RetinaEngine : public NodeEngine
{
    Q_OBJECT
    Q_PROPERTY(VideoSurface *  videoSurface READ videoSurface WRITE setVideoSurface NOTIFY videoSurfaceChanged)
    Q_PROPERTY(Kernel * kernel READ kernel WRITE setKernel NOTIFY kernelChanged)
    Q_PROPERTY(bool plotKernel READ plotKernel WRITE setPlotKernel NOTIFY plotKernelChanged)
    Q_PROPERTY(double sensitivity READ sensitivity WRITE setSensitivity NOTIFY sensitivityChanged)

public:
    RetinaEngine();
    ~RetinaEngine();

    void calculateFiringRate();


    VideoSurface * videoSurface() const;
    QImage paintedImage() const;
    Kernel * kernel() const;
    bool plotKernel() const;
    double sensitivity() const;


public slots:
    void receivedImage();
    void setVideoSurface(VideoSurface * videoSurface);
    void setKernel(Kernel * kernel);
    void setPlotKernel(bool plotKernel);

    void setSensitivity(double sensitivity);

signals:
    void videoSurfaceChanged(VideoSurface * videoSurface);
    void kernelChanged(Kernel * kernel);
    void plotKernelChanged(bool plotKernel);
    void sensitivityChanged(double sensitivity);

protected:
    virtual void stepEvent(double dt, bool parentEnabled) override;

private:
    VideoSurface * m_videoSurface = nullptr;
    Kernel * m_kernel = nullptr;
    QImage m_paintedImage;


    double m_firingRate = 0.0;
    double m_sensitivity = 1000.0;

    vector< vector <double>> m_stim;
    bool m_plotKernel;
};


#endif // RETINAENGINE_H
