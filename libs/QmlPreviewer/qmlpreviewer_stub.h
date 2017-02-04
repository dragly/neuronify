#ifndef QMLPREVIEWER_STUB_H
#define QMLPREVIEWER_STUB_H

#include <QObject>
#include <QVariant>
#include <QDebug>

class QmlPreviewer : public QObject
{
    Q_OBJECT
public:
    explicit QmlPreviewer(QApplication &app);

signals:

public slots:
    void reload(QString path) {}
    void show() {
        qFatal("Preview not supported on Android / iOS");
    }
    void handleDialogStart(QVariant qrcPaths, QUrl filePath) {}
};

#endif // QMLPREVIEWER_STUB_H
