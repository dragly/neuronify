#include <QApplication>
#include <QQmlApplicationEngine>
#include "neuronengine.h"

int main(int argc, char *argv[])
{
    qmlRegisterType<NeuronEngine>("Nestify", 1, 0, "NeuronEngine");
    QApplication app(argc, argv);

    QQmlApplicationEngine engine;
    engine.load(QUrl(QStringLiteral("qrc:///main.qml")));

    return app.exec();
}
