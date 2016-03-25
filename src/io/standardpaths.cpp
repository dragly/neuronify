#include "standardpaths.h"

#include <QDebug>

StandardPaths::StandardPaths(QObject *parent)
    : QObject(parent)
{

}

QString StandardPaths::writableLocation(StandardLocation location)
{
    return QStandardPaths::writableLocation((QStandardPaths::StandardLocation)location);
}

QObject* StandardPaths::qmlInstance(QQmlEngine *engine, QJSEngine *scriptEngine)
{
    Q_UNUSED(engine);
    Q_UNUSED(scriptEngine);

    return new StandardPaths;
}
