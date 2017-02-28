#include "qmlpreviewer.h"

#include <QDebug>
#include <QDirIterator>
#include <QRegularExpression>
#include <QProcess>
#include <QResource>
#include <QQmlEngine>
#include <QCryptographicHash>
#include <QTimer>

QmlPreviewer::QmlPreviewer(QGuiApplication &app)
    : m_app(app)
{
    Q_UNUSED(app)
    connect(&m_watcher, &QFileSystemWatcher::fileChanged, this, [&](const QString &path) {
        qDebug() << "Path" << path << "updated";
        if(m_reloadRequested) {
            qDebug() << "Reload already requested.";
            return;
        }
        m_reloadRequested = true;
        qDebug() << "Starting reload timer.";
        m_timer.singleShot(200, this, &QmlPreviewer::reload);
    });
}

void QmlPreviewer::reload()
{
    m_reloadRequested = false;
    m_view.engine()->clearComponentCache();

    qDebug() << "Reloading";

    for(auto qrcPath : m_qrcPaths) {
        QVariantMap map = qrcPath.toMap();
        qDebug() << "Unregistering" << map["path"].toString();
        QResource::unregisterResource(map["rcc"].toString(), m_prefix);
    }

    for(auto qrcPath : m_qrcPaths) {
        QVariantMap map = qrcPath.toMap();
        QProcess process;
        process.start("rcc", QStringList()
                      << "-binary" << map["path"].toUrl().toLocalFile()
                      << "-o" << map["rcc"].toString());
        process.waitForFinished();

        qDebug() << "Registering" << map["path"].toString();

        bool ok = QResource::registerResource(map["rcc"].toString(), m_prefix);
        if(!ok) {
            qWarning() << "Could not register resource for" << map["path"].toString() << map["rcc"].toString();
        }

        QDirIterator it(QString(":" + m_prefix), QStringList() << "*", QDir::Files, QDirIterator::Subdirectories);
        while (it.hasNext()) {
            QString next = it.next();
            QString relativeFilePath = next;
            QUrl qrcDirectory = map["path"].toUrl().adjusted(QUrl::RemoveFilename);
            relativeFilePath = relativeFilePath.replace(":" + m_prefix + "/", "");
//            qDebug() << "- Path" << qrcDirectory;
//            qDebug() << "- Relative" << relativeFilePath;
            QString result = qrcDirectory.resolved(QUrl(relativeFilePath)).toLocalFile();
//            qDebug() << "- Result" << result;
            if(QFileInfo::exists(result)) {
//                qDebug() << "-- Adding path" << result;
                m_watcher.addPath(result);
            }
        }

        qDebug() << "Watching" << m_watcher.files().count() << "files";
    }
    qDebug() << "Requesting QML to reload";
    QMetaObject::invokeMethod(m_rootItem, "reload");
}

void QmlPreviewer::setQrcPaths(QVariant qrcPaths)
{
    qDebug() << "Handle dialog start";

    for(auto qrcPath : m_qrcPaths) {
        QVariantMap map = qrcPath.toMap();
        qDebug() << "Unregistering" << map["path"].toString();
        QResource::unregisterResource(map["rcc"].toString(), m_prefix);
    }

    QVariantList paths = qrcPaths.toList();
    QUrl projectPath;
    m_qrcPaths.clear();
    QString proPaths = QMLPREVIEWER_RESOURCES;
    for(const QString &path : proPaths.split(" ")) {
        if(!path.endsWith("qmlpreviewer.qrc")) {
            qDebug() << "Propath" << path;
            paths.append(QUrl::fromLocalFile(path));
        }
    }

    for(QVariant path : paths) {
        QUrl pathUrl = path.toUrl();
        QString hash = QCryptographicHash::hash(pathUrl.toString().toLatin1(), QCryptographicHash::Md5).toBase64().replace("=", "").replace("/", "").replace("\\", "");
        QVariantMap map{
            {"path", pathUrl},
            {"rcc", hash + QString(".rcc")},
            {"hash", hash}
        };
        m_qrcPaths.append(map);
        qDebug() << "URL" << pathUrl;
        projectPath = path.toUrl().adjusted(QUrl::RemoveFilename);
    }
    reload();
    QMetaObject::invokeMethod(m_rootItem, "refreshFileView");
}

bool QmlPreviewer::show()
{
    if(m_app.arguments().contains("--qmlpreviewer")) {
        m_view.setTitle("QmlPreviewer");
        m_view.setSource(QUrl("qrc:///QmlPreviewer/QmlPreviewerDialog.qml"));
        m_view.setResizeMode(QQuickView::SizeRootObjectToView);
        m_rootItem = m_view.rootObject();
        connect(m_rootItem, SIGNAL(changeQrcPaths(QVariant)), this, SLOT(setQrcPaths(QVariant)));
        QMetaObject::invokeMethod(m_rootItem, "refresh");
        m_view.show();
        return true;
    }
    return false;
}

int QmlPreviewer::exec()
{
    return m_app.exec();
}
