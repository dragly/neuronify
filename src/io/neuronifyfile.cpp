#include "neuronifyfile.h"

#include <QtSql>
#include <QQuickItemGrabResult>

NeuronifyFile::NeuronifyFile(QObject *parent) : QObject(parent)
{

}

bool NeuronifyFile::save(const QUrl &fileUrl, const QString &name, const QString &description,
                         const QString &simulation, const QVariant &grabResult)
{
    QString filename = fileUrl.toLocalFile();

    QObject *grabObject = qvariant_cast<QObject*>(grabResult);
    QQuickItemGrabResult *grabItem = qobject_cast<QQuickItemGrabResult*>(grabObject);

    if(!grabItem) {
        qDebug() << "ERROR: NeuronifyFile::save: Could not get QQuickItemGrabResult.";
        return false;
    }

    qDebug() << "Saving" << filename;
    QFile originalFile(filename);
    if(originalFile.exists()) {
        originalFile.remove();
    }
    {
        QSqlDatabase db = QSqlDatabase::addDatabase("QSQLITE", filename);
        db.setDatabaseName(filename);
        db.open();

        QSqlQuery query = QSqlQuery(db);
        query.exec("CREATE TABLE IF NOT EXISTS simulations (name TEXT, description TEXT, data TEXT, screenshot BLOB)");

        QPixmap pixmap = QPixmap::fromImage(grabItem->image());
        QByteArray byteArray;
        QBuffer buffer(&byteArray);
        buffer.open(QIODevice::WriteOnly);
        pixmap.save(&buffer, "PNG");

        query.prepare( "INSERT INTO simulations (name, description, data, screenshot) VALUES (:name, :description, :data, :screenshot)" );
        query.bindValue(":name", name);
        query.bindValue(":description", description);
        query.bindValue(":data", simulation);
        query.bindValue(":screenshot", byteArray);

        if (!query.exec()) {
            qDebug() << "Error inserting image into table:\n" << query.lastError();
            return false;
        }
    }
    QSqlDatabase::removeDatabase(filename);
    return true;
}


QString NeuronifyFile::open(QUrl fileUrl)
{
    QString filename = fileUrl.toLocalFile();

    QString data;
    {
        QSqlDatabase db = QSqlDatabase::addDatabase("QSQLITE", filename);
        db.setDatabaseName(filename);
        db.open();
        QSqlQuery query = QSqlQuery(db);
        if(!query.exec("SELECT data FROM simulations")) {
            qDebug() << "ERROR loading simulations";
            return "";
        }
        query.first();

        data = query.value("data").toString();
    }
    QSqlDatabase::removeDatabase(filename);
    qDebug() << "NeuronifyFile::open: Returning data";
    return data;
}

QObject* NeuronifyFile::qmlInstance(QQmlEngine *engine, QJSEngine *scriptEngine)
{
    Q_UNUSED(engine);
    Q_UNUSED(scriptEngine);

    return new NeuronifyFile;
}
