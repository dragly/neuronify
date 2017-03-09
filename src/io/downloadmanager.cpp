#include "downloadmanager.h"

#include <QCoreApplication>
#include <QFile>
#include <QFileInfo>
#include <QList>
#include <QNetworkAccessManager>
#include <QNetworkRequest>
#include <QNetworkReply>
#include <QSslError>
#include <QStringList>
#include <QTimer>
#include <QUrl>
#include <QDir>
#include <QJSValue>
#include <QJSEngine>
#include <QQmlFile>

DownloadManager::DownloadManager()
{
    connect(&m_downloadManager, SIGNAL(finished(QNetworkReply*)),
            SLOT(downloadFinished(QNetworkReply*)));
    connect(&m_uploadManager, SIGNAL(finished(QNetworkReply*)),
            SLOT(uploadFinished(QNetworkReply*)));
}

void DownloadManager::upload(const QUrl localUrl, const QUrl &remoteUrl, QJSValue callback)
{
    QString filename = QQmlFile::urlToLocalFileOrQrc(localUrl);
    QFile file(filename);
    file.open(QFile::ReadOnly);

    QNetworkRequest request(remoteUrl);
    request.setRawHeader("X-Parse-Application-Id", "JffGes20AXUtdN9B6E1RkkHaS7DOxVmxJFSJgLoN");
    request.setRawHeader("X-Parse-REST-API-Key", "bBKStu7bqeyWFTYFfM5OIes255k9XEz2Voe4fUxS");
    request.setRawHeader("Content-Type", "image/png");

    QNetworkReply *reply = m_uploadManager.post(request, file.readAll());

    currentUploads[reply] = callback;
}

void DownloadManager::download(const QUrl &remoteUrl, const QUrl &localUrl)
{
    if(!localUrl.isLocalFile()) {
        qWarning() << "ERROR: Requested download location is not local url:" << localUrl;
        return;
    }
    QString targetFilename = localUrl.toLocalFile();
    qDebug() << "Download requested" << remoteUrl << targetFilename;
    QNetworkRequest request(remoteUrl);
    QNetworkReply *reply = m_downloadManager.get(request);

#ifndef QT_NO_SSL
    connect(reply, SIGNAL(sslErrors(QList<QSslError>)), SLOT(sslErrors(QList<QSslError>)));
#endif

    currentDownloads[reply] = targetFilename;
}

bool saveToDisk(const QString &filename, QIODevice *data)
{
    QFile file(filename);
    if (!file.open(QIODevice::WriteOnly)) {
        qWarning() << "ERROR: Could not open for writing:" << filename;
        return false;
    }

    file.write(data->readAll());
    file.close();

    return true;
}

void DownloadManager::sslErrors(const QList<QSslError> &sslErrors)
{
#ifndef QT_NO_SSL
    foreach (const QSslError &error, sslErrors)
        fprintf(stderr, "SSL error: %s\n", qPrintable(error.errorString()));
#else
    Q_UNUSED(sslErrors);
#endif
}

void DownloadManager::uploadFinished(QNetworkReply *reply)
{
    if(!currentUploads.contains(reply)) {
        qDebug() << "WARNING: DownloadManager found non-existent reply.";
        return;
    }

    QJSValue callback = currentUploads[reply];
    QJSValue resultArgument = QJSValue(QString::fromUtf8(reply->readAll()));

    callback.call(QJSValueList{resultArgument});

    currentUploads.remove(reply);
    reply->deleteLater();
}

void DownloadManager::downloadFinished(QNetworkReply *reply)
{
    if(!currentDownloads.contains(reply)) {
        qDebug() << "WARNING: DownloadManager found non-existent reply.";
        return;
    }
    QString filename = currentDownloads[reply];
    QDir targetDir = QFileInfo(filename).absoluteDir();
    if(!targetDir.mkpath(".")) {
        qDebug() << "ERROR: Could not create path" << targetDir;
        return;
    }
    QUrl url = reply->url();
    if (reply->error()) {
        qDebug() << "Download of %s failed:" << url << reply->errorString();
    } else {
        if (saveToDisk(filename, reply))
            qDebug() << "Download succeeded:" << url << filename;
    }

    currentDownloads.remove(reply);
    reply->deleteLater();
}
