#include <QApplication>
#include <QQmlApplicationEngine>
#include <QTextStream>
#include "neuronengine.h"
#include "fileio.h"

int main(int argc, char *argv[])
{
    qmlRegisterType<NeuronEngine>("Nestify", 1, 0, "NeuronEngine");
    QApplication app(argc, argv);

    qmlRegisterType<FileIO>("Nestify", 1, 0, "FileIO");



    QQmlApplicationEngine engine;
    engine.load(QUrl(QStringLiteral("qrc:///main.qml")));

    return app.exec();
}
