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

DownloadManager::DownloadManager()
{
    connect(&m_downloadManager, SIGNAL(finished(QNetworkReply*)),
            SLOT(downloadFinished(QNetworkReply*)));
    connect(&m_uploadManager, SIGNAL(finished(QNetworkReply*)),
            SLOT(uploadFinished(QNetworkReply*)));
}

void DownloadManager::upload(const QString filename, const QUrl &url, QJSValue callback)
{
    QFile file(filename);
    file.open(QFile::ReadOnly);

    QNetworkRequest request(url);
    request.setRawHeader("X-Parse-Application-Id", "JffGes20AXUtdN9B6E1RkkHaS7DOxVmxJFSJgLoN");
    request.setRawHeader("X-Parse-REST-API-Key", "bBKStu7bqeyWFTYFfM5OIes255k9XEz2Voe4fUxS");
    request.setRawHeader("Content-Type", "image/png");

    QNetworkReply *reply = m_uploadManager.post(request, file.readAll());

    currentUploads[reply] = callback;
}

void DownloadManager::download(const QUrl &url, const QString &targetLocation, const QString &targetFilename)
{
    qDebug() << "Download requested" << url << targetLocation;
    QNetworkRequest request(url);
    QNetworkReply *reply = m_downloadManager.get(request);

#ifndef QT_NO_SSL
    connect(reply, SIGNAL(sslErrors(QList<QSslError>)), SLOT(sslErrors(QList<QSslError>)));
#endif

    currentDownloads[reply] = {targetLocation, targetFilename};
}

QString generateFilename(const QUrl &url)
{
    QString path = url.path();
    QString basename = QFileInfo(path).fileName();

    if (basename.isEmpty())
        basename = "download";

    if (QFile::exists(basename)) {
        // already exists, don't overwrite
        int i = 0;
        basename += '.';
        while (QFile::exists(basename + QString::number(i)))
            ++i;

        basename += QString::number(i);
    }

    return basename;
}

bool saveToDisk(const QString &filename, QIODevice *data)
{
    QFile file(filename);
    if (!file.open(QIODevice::WriteOnly)) {
        fprintf(stderr, "Could not open %s for writing: %s\n",
                qPrintable(filename),
                qPrintable(file.errorString()));
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
    TargetInfo targetInfo = currentDownloads[reply];
    QString targetLocation = targetInfo.location;
    QString filename = targetInfo.filename;
    QDir targetDir(targetLocation);
    if(!targetDir.mkpath(".")) {
        qDebug() << "ERROR: Could not create path" << targetLocation;
        return;
    }
    QUrl url = reply->url();
    if (reply->error()) {
        qDebug() << "Download of %s failed:" << url << reply->errorString();
    } else {
        if(filename.isEmpty()) {
            filename = generateFilename(url);
        }
        QString targetFilePath = targetLocation + QDir::separator() + filename;
        if (saveToDisk(targetFilePath, reply))
            qDebug() << "Download succeeded:" << url << targetFilePath;
    }

    currentDownloads.remove(reply);
    reply->deleteLater();
}
