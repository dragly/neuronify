#include "neuronifyfile.h"

#include <QtSql>
#include <QQuickItemGrabResult>

NeuronifyFile::NeuronifyFile(QObject *parent) : QObject(parent)
{

}

QQuickItemGrabResult* itemFromGrabResult(const QVariant &grabResult) {
    if(!grabResult.isValid()) {
        return nullptr;
    }

    QObject *grabObject = qvariant_cast<QObject*>(grabResult);
    if(!grabObject) {
        return nullptr;
    }
    QQuickItemGrabResult *grabItem = qobject_cast<QQuickItemGrabResult*>(grabObject);
    return grabItem;
}

bool NeuronifyFile::save(const QUrl &fileUrl, const QString &name, const QString &description,
                         const QString &simulation, const QVariant &grabResult)
{
    QString filename = fileUrl.toLocalFile();

    qDebug() << Q_FUNC_INFO << "Saving" << name << "to" << filename;

    auto* grabItem = itemFromGrabResult(grabResult);
    QPixmap pixmap;
    if(grabItem) {
        pixmap = QPixmap::fromImage(grabItem->image());
    } else {
        qDebug() << Q_FUNC_INFO << "WARNING: No grab result, storing without screenshot";
    }

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


QVariant NeuronifyFile::open(QUrl fileUrl)
{
    QString filename = fileUrl.toLocalFile();

    QString data;
    QString name;
    QString description;
    {
        QSqlDatabase db = QSqlDatabase::addDatabase("QSQLITE", filename);
        db.setDatabaseName(filename);
        db.open();
        QSqlQuery query = QSqlQuery(db);
        if(!query.exec("SELECT name, description, data FROM simulations")) {
            qDebug() << "ERROR loading simulations";
            return "";
        }
        query.first();

        name = query.value("name").toString();
        description = query.value("description").toString();
        data = query.value("data").toString();
    }
    QSqlDatabase::removeDatabase(filename);
    qDebug() << "NeuronifyFile::open: Returning data";
    QVariantMap result;
    result["file"] = fileUrl;
    result["name"] = name;
    result["description"] = description;
    result["data"] = data;
    return result;
}

QObject* NeuronifyFile::qmlInstance(QQmlEngine *engine, QJSEngine *scriptEngine)
{
    Q_UNUSED(engine);
    Q_UNUSED(scriptEngine);

    return new NeuronifyFile;
}
