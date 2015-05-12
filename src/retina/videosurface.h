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
    Q_PROPERTY(bool enabled READ enabled WRITE setEnabled NOTIFY enabledChanged)

public:
    VideoSurface();
    ~VideoSurface();


    // QAbstractVideoSurface interface
    virtual QList<QVideoFrame::PixelFormat> supportedPixelFormats(QAbstractVideoBuffer::HandleType handleType) const;
    virtual bool present(const QVideoFrame &frame);

    QImage paintedImage() const;
    QObject * camera() const;

    bool enabled() const;

public slots:
    void setCamera(QObject *camera);
    void setEnabled(bool enabled);

signals:
    void gotImage(QRect paintedImage);
    void cameraChanged(QObject * camera);
    void enabledChanged(bool enabled);

private:
    QObject *m_camera = nullptr;
    QImage m_paintedImage;
    QVideoRendererControl *m_rendererControl = nullptr;
    QVideoProbe m_probe;
    int m_frameCounter = 0;
    bool m_enabled = true;
};



#endif // VIDEOSURFACE_H
