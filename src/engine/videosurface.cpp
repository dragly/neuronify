#include "videosurface.h"

#include <QDebug>
#include <QVideoSurfaceFormat>

VideoSurface::VideoSurface()
{

}

VideoSurface::~VideoSurface()
{

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

