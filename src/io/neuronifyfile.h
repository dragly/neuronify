#ifndef NEURONIFYFILE_H
#define NEURONIFYFILE_H

#include <QObject>
#include <QQmlEngine>
#include <QQuickItemGrabResult>

class NeuronifyFile : public QObject
{
    Q_OBJECT
public:
    explicit NeuronifyFile(QObject *parent = 0);

    Q_INVOKABLE QVariant open(QUrl fileUrl);
    static QObject* qmlInstance(QQmlEngine *engine, QJSEngine *scriptEngine);
    Q_INVOKABLE bool save(const QUrl &fileUrl, const QString &name, const QString &description, const QString &simulation, const QVariant &grabResult = QVariant());
};

#endif // NEURONIFYFILE_H
