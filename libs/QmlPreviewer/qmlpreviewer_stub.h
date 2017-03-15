#ifndef QMLPREVIEWER_STUB_H
#define QMLPREVIEWER_STUB_H

#include <QObject>
#include <QVariant>
#include <QDebug>
#include <QApplication>
#include <QUrl>

class QmlPreviewer : public QObject
{
    Q_OBJECT
public:
    explicit QmlPreviewer(QApplication &app) {
        Q_UNUSED(app)
    }

signals:

public slots:
    void reload(QString path) {
        Q_UNUSED(path)
    }
    bool show() {
        return true;
    }
    void exect() {
        return qFatal("Preview not supported on Android / iOS");
    }
    void handleDialogStart(QVariant qrcPaths, QUrl filePath) {
        Q_UNUSED(qrcPaths)
        Q_UNUSED(filePath)
    }
};

#endif // QMLPREVIEWER_STUB_H
