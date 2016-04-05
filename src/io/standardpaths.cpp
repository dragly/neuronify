#include "standardpaths.h"

#include <QDebug>

StandardPaths::StandardPaths(QObject *parent)
    : QObject(parent)
{

}

QUrl StandardPaths::writableLocation(StandardLocation location, const QString &fileName)
{
    QString path = QStandardPaths::writableLocation((QStandardPaths::StandardLocation)location);
    if(!fileName.isEmpty()) {
        path += "/" + fileName;
    }
    return QUrl::fromLocalFile(path);
}

QUrl StandardPaths::locate(StandardLocation location, const QString &fileName)
{
    return QUrl::fromLocalFile(QStandardPaths::locate((QStandardPaths::StandardLocation)location, fileName));
}

QString StandardPaths::toLocalFile(QUrl url)
{
    return url.toLocalFile();
}

QObject* StandardPaths::qmlInstance(QQmlEngine *engine, QJSEngine *scriptEngine)
{
    Q_UNUSED(engine);
    Q_UNUSED(scriptEngine);

    return new StandardPaths;
}
