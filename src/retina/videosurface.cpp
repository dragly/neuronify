#include "videosurface.h"

#include "androidmultimediautils.h"

#include <QDebug>
#include <QVideoSurfaceFormat>
#include <QVideoRendererControl>
#include <QCameraInfo>
#include <QPainter>

/*!
 * \class VideoSurface
 * \inmodule Neuronify
 * \ingroup neuronify-sensors
 * \brief Converts the camera frame to a gray-scale image.
 */

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

void VideoSurface::setEnabled(bool enabled)
{
    if (m_enabled == enabled)
        return;

    m_enabled = enabled;
    emit enabledChanged(enabled);
}


QList<QVideoFrame::PixelFormat> VideoSurface::supportedPixelFormats(QAbstractVideoBuffer::HandleType handleType) const
{
    Q_UNUSED(handleType);
    QList<QVideoFrame::PixelFormat> pixelFormat;
    pixelFormat.append(QVideoFrame::Format_RGB24);
    pixelFormat.append(QVideoFrame::Format_RGB32);
    pixelFormat.append(QVideoFrame::Format_NV21);
    pixelFormat.append(QVideoFrame::Format_ARGB32);
    pixelFormat.append(QVideoFrame::Format_Invalid);
    pixelFormat.append(QVideoFrame::Format_ARGB32);
    pixelFormat.append(QVideoFrame::Format_ARGB32_Premultiplied);
    pixelFormat.append(QVideoFrame::Format_RGB32);
    pixelFormat.append(QVideoFrame::Format_RGB24);
    pixelFormat.append(QVideoFrame::Format_RGB565);
    pixelFormat.append(QVideoFrame::Format_RGB555);
    pixelFormat.append(QVideoFrame::Format_ARGB8565_Premultiplied);
    pixelFormat.append(QVideoFrame::Format_BGRA32);
    pixelFormat.append(QVideoFrame::Format_BGRA32_Premultiplied);
    pixelFormat.append(QVideoFrame::Format_BGR32);
    pixelFormat.append(QVideoFrame::Format_BGR24);
    pixelFormat.append(QVideoFrame::Format_BGR565);
    pixelFormat.append(QVideoFrame::Format_BGR555);
    pixelFormat.append(QVideoFrame::Format_BGRA5658_Premultiplied);
    pixelFormat.append(QVideoFrame::Format_AYUV444);
    pixelFormat.append(QVideoFrame::Format_AYUV444_Premultiplied);
    pixelFormat.append(QVideoFrame::Format_YUV444);
    pixelFormat.append(QVideoFrame::Format_YUV420P);
    pixelFormat.append(QVideoFrame::Format_YV12);
    pixelFormat.append(QVideoFrame::Format_UYVY);
    pixelFormat.append(QVideoFrame::Format_YUYV);
    pixelFormat.append(QVideoFrame::Format_NV12);
    pixelFormat.append(QVideoFrame::Format_NV21);
    pixelFormat.append(QVideoFrame::Format_IMC1);
    pixelFormat.append(QVideoFrame::Format_IMC2);
    pixelFormat.append(QVideoFrame::Format_IMC3);
    pixelFormat.append(QVideoFrame::Format_IMC4);
    pixelFormat.append(QVideoFrame::Format_Y8);
    pixelFormat.append(QVideoFrame::Format_Y16);
    pixelFormat.append(QVideoFrame::Format_Jpeg);
    pixelFormat.append(QVideoFrame::Format_CameraRaw);
    pixelFormat.append(QVideoFrame::Format_AdobeDng);

    return pixelFormat;
}

bool VideoSurface::present(const QVideoFrame &constFrame)
{
    // TODO read only the center square of the image (it is shown as a square in QML anyways)

    if(!m_enabled) {
        return true;
    }
    QVideoFrame frame = constFrame;
#ifdef Q_OS_ANDROID
    if((m_frameCounter % 2) == 0) {
        frame.map(QAbstractVideoBuffer::ReadOnly);
        QSize frameSize = frame.size();
        int factor = 8;
        QSize newSize = QSize(frame.size().width() / factor, frame.size().height() / factor);
        QImage result(newSize, QImage::Format_ARGB32);
        qt_convert_NV21_to_ARGB32_grayscale_factor((const uchar *)frame.bits(),
                                  (quint32 *)result.bits(),
                                  frameSize.width(),
                                  frameSize.height(),
                                         factor);
        m_paintedImage = result;
        frame.unmap();
        emit gotImage(QRect());
    }
    m_frameCounter += 1;
    return true;
#else
    QVideoFrame myFrame = frame;
    myFrame.map(QAbstractVideoBuffer::ReadOnly);

    QImage::Format imageFormat = QVideoFrame::imageFormatFromPixelFormat(frame.pixelFormat());

    m_paintedImage = QImage(myFrame.bits(), myFrame.width(), myFrame.height(),
                     myFrame.bytesPerLine(), imageFormat);
#ifdef Q_OS_WIN
    // flip image because webcam data is messed up on Windows...
    QImage flipped(m_paintedImage.width(), m_paintedImage.height(), m_paintedImage.format());
    QPainter painter(&flipped);
    QTransform transf = painter.transform();
    transf.scale(1, -1);
    painter.setTransform(transf);
    painter.drawImage(0, -m_paintedImage.height(), m_paintedImage);
    m_paintedImage = flipped;
#endif

#ifdef Q_OS_MAC
    // flip image because webcam data is messed up on mac as well...
    m_paintedImage = m_paintedImage.mirrored(true, false);
#endif


    emit gotImage(QRect());
    return true;
#endif
}

QImage VideoSurface::paintedImage() const
{
    return m_paintedImage;
}

QObject *VideoSurface::camera() const
{
    return m_camera;
}

bool VideoSurface::enabled() const
{
    return m_enabled;
}

