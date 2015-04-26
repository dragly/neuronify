#ifndef VIDEOSURFACE_H
#define VIDEOSURFACE_H

#include <QAbstractVideoSurface>

class VideoSurface :  public QAbstractVideoSurface
{
    Q_OBJECT

public:
    VideoSurface();
    ~VideoSurface();


    // QAbstractVideoSurface interface
    virtual QList<QVideoFrame::PixelFormat> supportedPixelFormats(QAbstractVideoBuffer::HandleType handleType) const;
    virtual bool present(const QVideoFrame &frame);

    QImage image() const;


signals:
    void gotImage(QRect image);

private:
    QImage m_image;

};



#endif // VIDEOSURFACE_H
