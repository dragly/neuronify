#include <QApplication>
#include <QQmlApplicationEngine>
#include <QTextStream>
#include "engine/neuronengine.h"
#include "io/fileio.h"

int main(int argc, char *argv[])
{
    qmlRegisterType<NeuronEngine>("Neuronify", 1, 0, "NeuronEngine");
    QApplication app(argc, argv);

    qmlRegisterType<FileIO>("Neuronify", 1, 0, "FileIO");

    QQmlApplicationEngine engine;
    engine.load(QUrl(QStringLiteral("qrc:///main.qml")));

    return app.exec();
}
