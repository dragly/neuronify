#include "core/nodebase.h"
#include "core/nodeengine.h"
#include "core/edge.h"
#include "core/graphengine.h"

#include "retina/kernel.h"
#include "retina/kernels/gaborkernelengine.h"
#include "retina/kernels/dogkernelengine.h"
#include "retina/kernels/offleftkernelengine.h"
#include "retina/kernels/offrightkernelengine.h"
#include "retina/kernels/offbottomkernelengine.h"
#include "retina/kernels/offtopkernelengine.h"

#include "retina/retinaengine.h"
#include "retina/retinapainter.h"
#include "retina/videosurface.h"

#include "neurons/neuronengine.h"
#include "neurons/rateengine.h"
#include "neurons/current.h"
#include "neurons/passivecurrent.h"
#include "neurons/adaptationcurrent.h"

#include "io/fileio.h"
#include "io/standardpaths.h"
#include "io/propertygroup.h"

#include <QApplication>
#include <QQmlApplicationEngine>
#include <QTextStream>
#include <QtQml>


int main(int argc, char *argv[])
{
    qmlRegisterType<FileIO>("Neuronify", 1, 0, "FileIO");
    qmlRegisterSingletonType<StandardPaths>("Neuronify", 1, 0, "StandardPaths", &StandardPaths::qmlInstance);

    qmlRegisterType<NodeBase>("Neuronify", 1, 0, "NodeBase");
    qmlRegisterType<NodeEngine>("Neuronify", 1, 0, "NodeEngine");
    qmlRegisterType<Edge>("Neuronify", 1, 0, "Edge");
    qmlRegisterType<GraphEngine>("Neuronify", 1, 0, "GraphEngine");

    qmlRegisterType<NeuronEngine>("Neuronify", 1, 0, "NeuronEngine");

    qmlRegisterUncreatableType<AbstractKernelEngine>("Neuronify", 1, 0,
                                               "AbstractKernelEngine",
                                               "Derived classes need this");
    qmlRegisterType<GaborKernelEngine>("Neuronify", 1, 0, "GaborKernelEngine");
    qmlRegisterType<DogKernelEngine>("Neuronify", 1, 0, "DogKernelEngine");
    qmlRegisterType<OffLeftKernelEngine>("Neuronify", 1, 0, "OffLeftKernelEngine");
    qmlRegisterType<OffRightKernelEngine>("Neuronify", 1, 0, "OffRightKernelEngine");
    qmlRegisterType<OffTopKernelEngine>("Neuronify", 1, 0, "OffTopKernelEngine");
    qmlRegisterType<OffBottomKernelEngine>("Neuronify", 1, 0, "OffBottomKernelEngine");

    qmlRegisterType<Kernel>("Neuronify", 1, 0, "Kernel");
    qmlRegisterType<RetinaEngine>("Neuronify", 1, 0, "RetinaEngine");
    qmlRegisterType<RetinaPainter>("Neuronify", 1, 0, "RetinaPainter");
    qmlRegisterType<VideoSurface>("Neuronify", 1, 0, "VideoSurface");

    qmlRegisterType<Current>("Neuronify", 1, 0, "Current");
    qmlRegisterType<PassiveCurrent>("Neuronify", 1, 0, "PassiveCurrent");
    qmlRegisterType<AdaptationCurrent>("Neuronify", 1, 0, "AdaptationCurrent");

    qmlRegisterType<RateEngine>("Neuronify", 1, 0, "RateEngine");
    qmlRegisterType<PropertyGroup>("Neuronify", 1, 0, "PropertyGroup");

    QApplication app(argc, argv);
    app.setOrganizationName("Ovilab");
    app.setOrganizationDomain("net");
    app.setApplicationName("Neuronify");

    QQmlApplicationEngine engine;
    engine.load(QUrl(QStringLiteral("qrc:///qml/main.qml")));

    return app.exec();
}
