#ifndef FILEIO_H
#define FILEIO_H

#include <QObject>
#include <QUrl>

class FileIO : public QObject
{
    Q_OBJECT

public:
    Q_PROPERTY(QUrl source
               READ source
               WRITE setSource
               NOTIFY sourceChanged)
    explicit FileIO(QObject *parent = 0);

    Q_INVOKABLE QString read();
    Q_INVOKABLE bool write(const QString &data);

    QUrl source() { return mSource; }

public slots:
    void setSource(const QUrl& source) { mSource = source; }

signals:
    void sourceChanged(const QUrl& source);
    void error(const QString& msg);

private:
    QUrl mSource;
};

#endif // FILEIO_H
