#include "videosurface.h"

#include <QDebug>
#include <QVideoSurfaceFormat>
#include <QVideoRendererControl>

VideoSurface::VideoSurface()
{
    connect(&m_probe, &QVideoProbe::videoFrameProbed, this, &VideoSurface::present);

}

VideoSurface::~VideoSurface()
{

}

void VideoSurface::setCamera(QObject* camera)
{
    if (m_camera == camera)
        return;

    m_camera = camera;
    QCamera *cameraObject = qvariant_cast<QCamera *>(m_camera->property("mediaObject"));
    if(cameraObject) {
#ifdef Q_OS_ANDROID
        qDebug() << "Setting probe source";
        bool sourceSuccess = m_probe.setSource(cameraObject);
        if(!sourceSuccess) {
            qWarning() << "Could not set probe source!";
        }
#else
        qDebug() << "Setting renderer control";
        m_rendererControl = cameraObject->service()->requestControl<QVideoRendererControl *>();
        if(m_rendererControl) {
            qDebug() << "Setting renderer surface";
            m_rendererControl->setSurface(this);
        }
#endif
    }

    emit cameraChanged(camera);
}


QList<QVideoFrame::PixelFormat> VideoSurface::supportedPixelFormats(QAbstractVideoBuffer::HandleType handleType) const
{
    QList<QVideoFrame::PixelFormat> pixelFormat;
    pixelFormat.append(QVideoFrame::Format_RGB24);

    return pixelFormat;
}

bool VideoSurface::present(const QVideoFrame &frame)
{
#ifdef Q_OS_ANDROID
    qDebug() << "Present got frame" << frame;
#endif

    QVideoFrame myFrame = frame;
    myFrame.map(QAbstractVideoBuffer::ReadOnly);

    QImage::Format imageFormat = QVideoFrame::imageFormatFromPixelFormat(frame.pixelFormat());
    m_image = QImage(myFrame.bits(), myFrame.width(), myFrame.height(),
                     myFrame.bytesPerLine(), imageFormat);
    emit gotImage(QRect());
    return true;
}

QImage VideoSurface::image() const
{
    return m_image;
}

QObject *VideoSurface::camera() const
{
    return m_camera;
}

