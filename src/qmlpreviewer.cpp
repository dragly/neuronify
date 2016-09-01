#include "qmlpreviewer.h"

#include <QDebug>
#include <QApplication>
#include <QDirIterator>
#include <QRegularExpression>
#include <QProcess>
#include <QResource>
#include <QQmlEngine>

QmlPreviewer::QmlPreviewer(QApplication &app)
{
//    QStringList args = app.arguments();
//    if(args.count() < 4) {
//        qFatal("ERROR: Not enough arguments");
//    }
//    m_projectPath = args[1];
//    m_filePath = args[2];

    connect(&m_watcher, &QFileSystemWatcher::fileChanged, this, &QmlPreviewer::reload);
}

void QmlPreviewer::reload(QString path)
{
//    if(m_projectPath.isEmpty() || m_filePath.isEmpty()) {

//        QObject *item = m_view.rootObject();
//        connect(item, SIGNAL(start(QUrl, QUrl)), this, SLOT(handleDialogStart(QUrl,QUrl)));
//        return;
//    }

    m_view.engine()->clearComponentCache();

    QString prefix = "/qtqmlpreview";
    qDebug() << "Reloading" << m_filePath;
    QResource::unregisterResource("qml.rcc", prefix);

    QProcess process;
    process.start("rcc", QStringList()
                  << "-binary" << (m_projectPath + QString("/qml.qrc"))
                  << "-o" << "qml.rcc");
    process.waitForFinished();

    QResource::registerResource("qml.rcc", prefix);

    QMetaObject::invokeMethod(m_rootItem, "reload");

//    QFile file(m_filePath);
//    file.open(QFile::ReadOnly);
//    QVariant contents = file.readAll();
//    file.close();

//    QString filePathClipped = m_filePath;
//    filePathClipped.replace(QRegularExpression(QString("^") + m_projectPath), "");

//    QUrl url("qrc:///" + prefix + "/" + filePathClipped);
//    m_view.setSource(QUrl());
//    m_view.engine()->clearComponentCache();
//    m_view.setSource(url);
//    QQmlComponent component(m_view.engine());
//    component.setData(contents.toByteArray(), url);
//    if(m_object) {
//        m_object->deleteLater();
//    }
//    m_object = component.create();
//    m_view.setContent(url, &component, m_object);

    if(!path.isEmpty()) {
        m_watcher.addPath(path);
    }
}

void QmlPreviewer::handleDialogStart(QUrl projectPath, QUrl filePath)
{
    m_projectPath = projectPath.toLocalFile();
    m_filePath = filePath.toLocalFile();

    QDirIterator it(m_projectPath, QStringList() << "*", QDir::Files, QDirIterator::Subdirectories);
    while (it.hasNext()) {
        QString next = it.next();
        m_watcher.addPath(next);
    }

    reload(m_filePath);
}

void QmlPreviewer::show()
{
    m_view.setSource(QUrl("qrc:///QmlPreviewerDialog.qml"));
    m_view.setResizeMode(QQuickView::SizeViewToRootObject);
    m_view.show();

    m_rootItem = m_view.rootObject();
    connect(m_rootItem, SIGNAL(start(QUrl, QUrl)), this, SLOT(handleDialogStart(QUrl,QUrl)));
}
