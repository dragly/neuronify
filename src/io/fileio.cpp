
#include "fileio.h"
#include <QFile>
#include <QTextStream>
#include <QDebug>
#include <QQmlFile>
#include <QDir>

/*!
 * \class FileIO
 * \inmodule Neuronify
 * \ingroup neuronify-utils
 * \brief FileIO is a helper class used for reading and writing files from QML.
 *
 * This is mainly used to read and save simulation files.
 */

FileIO::FileIO(QObject *parent) :
    QObject(parent)
{

}

QString FileIO::read()
{
    if (m_source.isEmpty()){
        emit error("source is empty");
        return QString();
    }

    QString path = QQmlFile::urlToLocalFileOrQrc(m_source);

    QFile file(path);
    QString fileContent;
    if ( file.open(QIODevice::ReadOnly) ) {
        QString line;
        QTextStream t( &file );
        do {
            line = t.readLine();
            fileContent += line + "\n";
        } while (!line.isNull());
        file.close();
    } else {
        emit error("Unable to open the file " + path);
        return QString();
    }

    return fileContent;
}

bool FileIO::write(const QString& data)
{
    if (m_source.isEmpty()){
        qDebug() << "Source is empty!";
        return false;
    }
    QString path = QQmlFile::urlToLocalFileOrQrc(m_source);
    QFileInfo fileinfo(path);
    QDir directory = fileinfo.absoluteDir();
    if(!directory.exists() && !directory.mkpath(".")) {
        qDebug() << "Cannot make path to file" << m_source;
        return false;
    }
    QFile file(path);
    if (!file.open(QFile::WriteOnly | QFile::Truncate)){
        qDebug() << "Couldn't open file" << path;
        return false;
    }

    QTextStream out(&file);
    out << data;

    file.close();

    return true;
}
