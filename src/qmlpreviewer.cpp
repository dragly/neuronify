#include "qmlpreviewer.h"

#include <QDebug>
#include <QApplication>
#include <QDirIterator>
#include <QRegularExpression>
#include <QProcess>
#include <QResource>
#include <QQmlEngine>
#include <QCryptographicHash>

QmlPreviewer::QmlPreviewer(QApplication &app)
{
    connect(&m_watcher, &QFileSystemWatcher::fileChanged, this, &QmlPreviewer::reload);
}

void QmlPreviewer::reload(QString path)
{

    m_view.engine()->clearComponentCache();

    qDebug() << "Reloading" << m_filePath;

    for(auto qrcPath : m_qrcPaths) {
        QVariantMap map = qrcPath.toMap();
        qDebug() << "Unregistering" << map["rcc"].toString();
        QResource::unregisterResource(map["rcc"].toString(), m_prefix);
    }


    for(auto qrcPath : m_qrcPaths) {
        QVariantMap map = qrcPath.toMap();
        QProcess process;
        process.start("rcc", QStringList()
                      << "-binary" << map["path"].toUrl().toLocalFile()
                      << "-o" << map["rcc"].toString());
        qDebug() << process.readAllStandardError();
        process.waitForFinished();
        qDebug() << process.readAllStandardError();

        qDebug() << "Registering" << map["rcc"].toString();

        QResource::registerResource(map["rcc"].toString(), m_prefix);
    }

    QMetaObject::invokeMethod(m_rootItem, "reload");

    if(!path.isEmpty()) {
        m_watcher.addPath(path);
    }
}

void QmlPreviewer::handleDialogStart(QUrl projectPath, QUrl filePath, QVariant qrcPaths)
{
    qDebug() << "Handle dialog start" << projectPath << filePath << qrcPaths;

    m_projectPath = projectPath;

    for(auto qrcPath : m_qrcPaths) {
        QVariantMap map = qrcPath.toMap();
        qDebug() << "Unregistering" << map["rcc"].toString();
        QResource::unregisterResource(map["rcc"].toString(), m_prefix);
    }

    QVariantList paths = qrcPaths.toList();
    m_qrcPaths.clear();
    for(QVariant path : paths) {
        QUrl pathUrl = path.toUrl();
        QString hash = QCryptographicHash::hash(pathUrl.toString().toLatin1(), QCryptographicHash::Md5).toBase64().replace("=", "");
        QVariantMap map{
            {"path", pathUrl},
            {"rcc", hash + QString(".rcc")},
            {"hash", hash}
        };
        m_qrcPaths.append(map);
    }
    m_filePath = filePath.toLocalFile();

    qDebug() << "Debug:" << m_projectPath.toLocalFile();

    QDirIterator it(m_projectPath.toLocalFile(), QStringList() << "*", QDir::Files, QDirIterator::Subdirectories);
    while (it.hasNext()) {
        QString next = it.next();
        qDebug() << "Watching" << next;
        m_watcher.addPath(next);
    }

    reload(m_filePath);
}

void QmlPreviewer::show()
{
    m_view.setSource(QUrl("qrc:///QmlPreviewerDialog.qml"));
    m_view.setResizeMode(QQuickView::SizeRootObjectToView);
    m_view.show();

    m_rootItem = m_view.rootObject();
    connect(m_rootItem, SIGNAL(start(QUrl, QUrl, QVariant)), this, SLOT(handleDialogStart(QUrl, QUrl, QVariant)));
}
