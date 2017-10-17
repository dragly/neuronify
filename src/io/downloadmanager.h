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

struct DownloadData {
    QString filename;
    QJSValue callback;
};

class DownloadManager: public QObject
{
    Q_OBJECT

public:
    DownloadManager();
    Q_INVOKABLE void download(const QUrl &remoteUrl, const QUrl &localUrl, const QString &token, QJSValue callback);
    Q_INVOKABLE void download(const QString &bucket, const QString &objectName, const QUrl &localUrl, const QString &token, QJSValue callback);
    Q_INVOKABLE void upload(const QUrl localUrl, const QUrl &remoteUrl, const QString &token, QJSValue callback);

private slots:
    void downloadFinished(QNetworkReply *reply);
    void uploadFinished(QNetworkReply *reply);
    void sslErrors(const QList<QSslError> &errors);

private:
    QNetworkAccessManager m_downloadManager;
    QNetworkAccessManager m_uploadManager;
    QMap<QNetworkReply*, DownloadData> currentDownloads;
    QMap<QNetworkReply*, QJSValue> currentUploads;
};

#endif // DOWNLOADMANAGER_H
