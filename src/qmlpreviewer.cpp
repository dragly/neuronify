#include "qmlpreviewer.h"

#include <QDebug>
#include <QApplication>
#include <QDirIterator>
#include <QRegularExpression>

QmlPreviewer::QmlPreviewer(QApplication &app)
{
    QStringList args = app.arguments();
    if(args.count() < 3) {
        qFatal("ERROR: Not enough arguments");
    }
    m_projectPath = args[1];
    m_filePath = args[2];
}

void QmlPreviewer::reload(QString path)
{
    qDebug() << "Reloading" << m_filePath;

    QFile file(m_filePath);
    file.open(QFile::ReadOnly);
    QVariant contents = file.readAll();
    file.close();

    QString filePathClipped = m_filePath;
    filePathClipped.replace(QRegularExpression(QString("^") + m_projectPath), "");

    QQmlComponent component(m_view.engine());
    QUrl url("qrc:///" + filePathClipped);
    component.setData(contents.toByteArray(), url);
    if(m_object) {
        m_object->deleteLater();
    }
    m_object = component.create();
    m_view.setContent(url, &component, m_object);
    m_watcher.addPath(path);
}

void QmlPreviewer::show()
{
    m_view.setResizeMode(QQuickView::SizeViewToRootObject);
    m_view.show();
    m_rootItem = m_view.rootObject();

    QDirIterator it(m_projectPath, QStringList() << "*", QDir::Files, QDirIterator::Subdirectories);
    while (it.hasNext()) {
        m_watcher.addPath(it.next());
    }

    connect(&m_watcher, &QFileSystemWatcher::fileChanged, this, &QmlPreviewer::reload);

    reload(m_filePath);
}
