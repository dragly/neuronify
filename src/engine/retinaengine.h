#ifndef RETINAENGINE_H
#define RETINAENGINE_H

#include <QCamera>
#include <iostream>
#include <vector>
#include <math.h>
#include <QVideoRendererControl>
#include <QVideoProbe>

#include "videosurface.h"
#include "nodeengine.h"

using namespace std;

class RetinaEngine : public NodeEngine
{
    Q_OBJECT
    Q_PROPERTY(QObject* camera READ camera WRITE setCamera NOTIFY cameraChanged)

public:
    RetinaEngine();
    ~RetinaEngine();

    void calculateFiringRate();
    double temporalRF(const double tau);
    double gaborField(int x, int y);

    QImage image() const;
    QObject* camera() const;

public slots:
    void receivedImage();
    void setCamera(QObject* camera);

signals:
    void cameraChanged(QObject* camera);

protected:
    virtual void stepEvent(double dt);

private:
    QObject* m_camera = nullptr;
    QCamera* m_cameraObject = nullptr;
    VideoSurface m_videoSurface;
    QImage m_image;
    QVideoProbe m_probe;

    vector< vector <double>> m_stim;
    vector< vector <double>> m_recField;

    int m_nPixelsX;
    int m_nPixelsY;
    double m_firingRate = 0.0;

    void makeReceptiveField();

    QVideoRendererControl* m_rendererControl;

};


#endif // RETINAENGINE_H
