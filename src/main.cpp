#include "engine/node.h"
#include "engine/edge.h"
#include "engine/neuronnode.h"
#include "engine/current.h"
#include "engine/graphengine.h"
#include "currents/passivecurrent.h"
#include "currents/adaptationcurrent.h"
#include "io/fileio.h"

#include <QApplication>
#include <QQmlApplicationEngine>
#include <QTextStream>
#include <QtQml>

int main(int argc, char *argv[])
{
    qmlRegisterType<FileIO>("Neuronify", 1, 0, "FileIO");

    qmlRegisterType<Node>("Neuronify", 1, 0, "Node");
    qmlRegisterType<Edge>("Neuronify", 1, 0, "Edge");
    qmlRegisterType<GraphEngine>("Neuronify", 1, 0, "GraphEngine");

    qmlRegisterType<NeuronNode>("Neuronify", 1, 0, "NeuronNode");

    qmlRegisterType<Current>("Neuronify", 1, 0, "Current");
    qmlRegisterType<PassiveCurrent>("Neuronify", 1, 0, "PassiveCurrent");
    qmlRegisterType<AdaptationCurrent>("Neuronify", 1, 0, "AdaptationCurrent");

    QApplication app(argc, argv);

    QQmlApplicationEngine engine;
    engine.load(QUrl(QStringLiteral("qrc:///main.qml")));

    return app.exec();
}
