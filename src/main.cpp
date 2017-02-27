#include <CuteVersioning/CuteVersioning>

#include "core/nodebase.h"
#include "core/nodeengine.h"
#include "core/edgebase.h"
#include "core/graphengine.h"
#include "core/edgeengine.h"

#include "retina/kernel.h"
#include "retina/kernels/gaborkernelengine.h"
#include "retina/kernels/dogkernelengine.h"
#include "retina/kernels/rectangularkernelengine.h"

#include "retina/retinaengine.h"
#include "retina/retinapainter.h"
#include "retina/videosurface.h"

#include "neurons/neuronengine.h"
#include "neurons/rateengine.h"
#include "neurons/current.h"
#include "neurons/leakcurrent.h"
#include "neurons/adaptationcurrent.h"

#include "io/fileio.h"
#include "io/standardpaths.h"
#include "io/propertygroup.h"

#include <QApplication>
#include <QQmlApplicationEngine>
#include <QTextStream>
#include <QtQml>
#include <QQmlContext>
#include <QQuickView>
#include <QDebug>
#include <QmlPreviewer>

int main(int argc, char *argv[])
{
    qint64 startupTime = QDateTime::currentMSecsSinceEpoch();
    qDebug() << "Neuronify version" << CuteVersioning::identifier << "started at" << startupTime;

    CuteVersioning::init();

    qmlRegisterType<FileIO>("Neuronify", 1, 0, "FileIO");
    qmlRegisterSingletonType<StandardPaths>("Neuronify", 1, 0, "StandardPaths", &StandardPaths::qmlInstance);

    qmlRegisterType<NodeBase>("Neuronify", 1, 0, "NodeBase");
    qmlRegisterType<NodeEngine>("Neuronify", 1, 0, "NodeEngine");
    qmlRegisterType<EdgeBase>("Neuronify", 1, 0, "EdgeBase");
    qmlRegisterType<GraphEngine>("Neuronify", 1, 0, "GraphEngine");
    qmlRegisterType<EdgeEngine>("Neuronify", 1, 0, "EdgeEngine");

    qmlRegisterType<NeuronEngine>("Neuronify", 1, 0, "NeuronEngineBase");

    qmlRegisterUncreatableType<AbstractKernelEngine>("Neuronify", 1, 0,
                                               "AbstractKernelEngine",
                                               "Derived classes need this");
    qmlRegisterType<GaborKernelEngine>("Neuronify", 1, 0, "GaborKernelEngine");
    qmlRegisterType<DogKernelEngine>("Neuronify", 1, 0, "DogKernelEngine");
    qmlRegisterType<RectangularKernelEngine>("Neuronify", 1, 0,
                                             "RectangularKernelEngine");


    qmlRegisterType<Kernel>("Neuronify", 1, 0, "Kernel");
    qmlRegisterType<RetinaEngine>("Neuronify", 1, 0, "RetinaEngine");
    qmlRegisterType<RetinaPainter>("Neuronify", 1, 0, "RetinaPainter");
    qmlRegisterType<VideoSurface>("Neuronify", 1, 0, "VideoSurface");

    qmlRegisterType<Current>("Neuronify", 1, 0, "Current");
    qmlRegisterType<LeakCurrent>("Neuronify", 1, 0, "LeakCurrent");
    qmlRegisterType<AdaptationCurrent>("Neuronify", 1, 0, "AdaptationCurrent");

    qmlRegisterType<RateEngine>("Neuronify", 1, 0, "RateEngine");
    qmlRegisterType<PropertyGroup>("Neuronify", 1, 0, "PropertyGroup");


    QApplication app(argc, argv);
    app.setOrganizationName("Ovilab");
    app.setOrganizationDomain("net");
    app.setApplicationName("Neuronify");

    QmlPreviewer previewer(app);
    if(previewer.show()) {
        return previewer.exec();
    }

    QQmlApplicationEngine engine;
    qDebug() << "Making" << QStandardPaths::writableLocation(QStandardPaths::AppConfigLocation) << "/savedata";
    QDir().mkpath(QStandardPaths::writableLocation(QStandardPaths::AppConfigLocation) + "/savedata");

    engine.load(QUrl(QStringLiteral("qrc:///qml/main.qml")));
    if(engine.rootObjects().size() > 0) {
        QVariant qmlStartupTime = QQmlProperty::read(engine.rootObjects().first(), "startupTime");
        qDebug() << "Load time:" << qmlStartupTime.toDouble() - startupTime;
    }
    return app.exec();
}
