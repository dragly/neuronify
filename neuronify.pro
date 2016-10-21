TEMPLATE = app

TARGET = neuronify

android {
    TARGET = Neuronify
}

QT += qml quick widgets multimedia multimediawidgets charts

CONFIG += c++11 qtquickcompiler

HEADERS += \
    src/io/fileio.h \
    src/core/graphengine.h \
    src/core/nodebase.h \
    src/core/nodeengine.h \
    src/neurons/current.h \
    src/neurons/neuronengine.h \
    src/retina/androidmultimediautils.h \
    src/retina/retinaengine.h \
    src/retina/retinapainter.h \
    src/retina/videosurface.h \
    src/neurons/leakcurrent.h \
    src/neurons/adaptationcurrent.h \
    src/retina/kernel.h \
    src/retina/kernels/gaborkernelengine.h \
    src/retina/kernels/abstractkernelengine.h \
    src/retina/kernels/dogkernelengine.h \
    src/io/standardpaths.h \
    src/neurons/rateengine.h \
    src/utility/mathhelper.h \
    src/core/neuronifyobject.h \
    src/io/propertygroup.h \
    src/retina/kernels/rectangularkernelengine.h \
    src/core/edgebase.h \
    src/core/edgeengine.h \
    src/qmlpreviewer.h

SOURCES += \
    src/io/fileio.cpp \
    src/main.cpp \
    src/core/nodebase.cpp \
    src/core/nodeengine.cpp \
    src/core/graphengine.cpp \
    src/neurons/current.cpp \
    src/neurons/neuronengine.cpp \
    src/retina/retinaengine.cpp \
    src/retina/retinapainter.cpp \
    src/retina/videosurface.cpp \
    src/retina/androidmultimediautils.cpp \
    src/neurons/adaptationcurrent.cpp \
    src/neurons/leakcurrent.cpp \
    src/retina/kernel.cpp \
    src/retina/kernels/gaborkernelengine.cpp \
    src/retina/kernels/abstractkernelengine.cpp \
    src/retina/kernels/dogkernelengine.cpp \
    src/io/standardpaths.cpp \
    src/neurons/rateengine.cpp \
    src/utility/mathhelper.cpp \
    src/core/neuronifyobject.cpp \
    src/io/propertygroup.cpp \
    src/retina/kernels/rectangularkernelengine.cpp \
    src/core/edgebase.cpp \
    src/core/edgeengine.cpp \
    src/qmlpreviewer.cpp


RESOURCES += qml.qrc \
    images.qrc \
    simulations.qrc \
    sounds.qrc

# Additional import path used to resolve QML modules in Qt Creator's code model
QML_IMPORT_PATH =

# Default rules for deployment.
include(deployment.pri)

DISTFILES += \
    android/gradle/wrapper/gradle-wrapper.jar \
    android/AndroidManifest.xml \
    android/gradlew.bat \
    android/res/values/libs.xml \
    android/build.gradle \
    android/gradle/wrapper/gradle-wrapper.properties \
    android/gradlew \
    android/src/org/cinpla/neuronify/AlwaysOnActivity.java \
    qml/sensors/singletons/qmldir \
    COPYING.md \
    snapcraft.yaml

ANDROID_PACKAGE_SOURCE_DIR = $$PWD/android

exists(libs/CuteVersioning/CuteVersioning.pri) {
    GIT_DIR=$$PWD/.git
    GIT_WORK_TREE=$$PWD
    include(libs/CuteVersioning/CuteVersioning.pri)
} else {
    error("Could not find CuteVersioning. Try running 'git submodule update --init --recursive' in Neuronify's root directory.")
}
