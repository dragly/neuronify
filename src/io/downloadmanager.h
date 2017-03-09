#ifndef DOWNLOADMANAGER_H
#define DOWNLOADMANAGER_H

#include <QObject>
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
#include <memory>
#include <QJSValue>

class DownloadManager: public QObject
{
    Q_OBJECT

public:
    DownloadManager();
    Q_INVOKABLE void download(const QUrl &remoteUrl, const QUrl &localUrl);
    Q_INVOKABLE void upload(const QUrl localUrl, const QUrl &remoteUrl, QJSValue callback);

private slots:
    void downloadFinished(QNetworkReply *reply);
    void uploadFinished(QNetworkReply *reply);
    void sslErrors(const QList<QSslError> &errors);

private:
    QNetworkAccessManager m_downloadManager;
    QNetworkAccessManager m_uploadManager;
    QMap<QNetworkReply*, QString> currentDownloads;
    QMap<QNetworkReply*, QJSValue> currentUploads;
};

#endif // DOWNLOADMANAGER_H
