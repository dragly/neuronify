TEMPLATE = app

TARGET = neuronify

android {
    TARGET = Neuronify
}

QT += qml quick widgets multimedia multimediawidgets charts

CONFIG += c++11

HEADERS += \
    src/io/fileio.h \
    src/core/graphengine.h \
    src/core/nodebase.h \
    src/core/nodeengine.h \
    src/core/edge.h \
    src/neurons/current.h \
    src/neurons/neuronengine.h \
    src/retina/androidmultimediautils.h \
    src/retina/receptivefield.h \
    src/retina/retinaengine.h \
    src/retina/retinapainter.h \
    src/retina/videosurface.h \
    src/neurons/passivecurrent.h \
    src/neurons/adaptationcurrent.h

SOURCES += \
    src/io/fileio.cpp \
    src/main.cpp \
    src/core/nodebase.cpp \
    src/core/nodeengine.cpp \
    src/core/graphengine.cpp \
    src/core/edge.cpp \
    src/neurons/current.cpp \
    src/neurons/neuronengine.cpp \
    src/retina/retinaengine.cpp \
    src/retina/retinapainter.cpp \
    src/retina/videosurface.cpp \
    src/retina/receptivefield.cpp \
    src/retina/androidmultimediautils.cpp \
    src/neurons/adaptationcurrent.cpp \
    src/neurons/passivecurrent.cpp

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
    COPYING.md

ANDROID_PACKAGE_SOURCE_DIR = $$PWD/android
