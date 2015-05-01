#ifndef VIDEOSURFACE_H
#define VIDEOSURFACE_H

#include <QAbstractVideoSurface>
#include <QCamera>
#include <QVideoRendererControl>
#include <QVideoProbe>


class VideoSurface :  public QAbstractVideoSurface
{
    Q_OBJECT
    Q_PROPERTY(QObject *  camera READ camera WRITE setCamera NOTIFY cameraChanged)

public:
    VideoSurface();
    ~VideoSurface();


    // QAbstractVideoSurface interface
    virtual QList<QVideoFrame::PixelFormat> supportedPixelFormats(QAbstractVideoBuffer::HandleType handleType) const;
    virtual bool present(const QVideoFrame &frame);

    QImage image() const;
    QObject * camera() const;

public slots:
    void setCamera(QObject *camera);

signals:
    void gotImage(QRect image);
    void cameraChanged(QObject * camera);

private:
    QObject * m_camera;
    QImage m_image;
    QVideoRendererControl* m_rendererControl;
    QVideoProbe m_probe;
    int m_frameCounter = 0;
};



#endif // VIDEOSURFACE_H
