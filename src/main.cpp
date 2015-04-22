#include "engine/neuronnode.h"
#include "engine/conductance.h"
#include "engine/current.h"
#include "currents/passivecurrent.h"
#include "io/fileio.h"

#include <QApplication>
#include <QQmlApplicationEngine>
#include <QTextStream>
#include <QtQml>

int main(int argc, char *argv[])
{
    qmlRegisterType<NeuronNode>("Neuronify", 1, 0, "NeuronNode");
    qmlRegisterType<Conductance>("Neuronify", 1, 0, "Conductance");
    qmlRegisterType<Current>("Neuronify", 1, 0, "Current");
    qmlRegisterType<PassiveCurrent>("Neuronify", 1, 0, "PassiveCurrent");
    QApplication app(argc, argv);

    qmlRegisterType<FileIO>("Neuronify", 1, 0, "FileIO");

    QQmlApplicationEngine engine;
    engine.load(QUrl(QStringLiteral("qrc:///main.qml")));

    return app.exec();
}
