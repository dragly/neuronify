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

struct TargetInfo
{
    QString location;
    QString filename;
};

class DownloadManager: public QObject
{
    Q_OBJECT

public:
    DownloadManager();
    Q_INVOKABLE void download(const QUrl &url, const QString &targetLocation, const QString &targetFilename = "");
    Q_INVOKABLE void upload(const QString filename, const QUrl &url, QJSValue callback);

private slots:
    void downloadFinished(QNetworkReply *reply);
    void uploadFinished(QNetworkReply *reply);
    void sslErrors(const QList<QSslError> &errors);

private:
    QNetworkAccessManager m_downloadManager;
    QNetworkAccessManager m_uploadManager;
    QMap<QNetworkReply*, TargetInfo> currentDownloads;
    QMap<QNetworkReply*, QJSValue> currentUploads;
};

#endif // DOWNLOADMANAGER_H
