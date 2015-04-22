TEMPLATE = app

TARGET = neuronify

android {
    TARGET = Neuronify
}

QT += qml quick widgets

CONFIG += c++11

HEADERS += \
    src/io/fileio.h \
    src/engine/conductance.h \
    src/engine/node.h \
    src/engine/neuronnode.h \
    src/engine/entity.h \
    src/engine/current.h \
    src/currents/passivecurrent.h

SOURCES += \
    src/io/fileio.cpp \
    src/main.cpp \
    src/engine/conductance.cpp \
    src/engine/node.cpp \
    src/engine/neuronnode.cpp \
    src/engine/entity.cpp \
    src/engine/current.cpp \
    src/currents/passivecurrent.cpp

RESOURCES += qml/qml.qrc \
    images/images.qrc \
    simulations/simulations.qrc

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
    android/src/org/cinpla/neuronify/AlwaysOnActivity.java

ANDROID_PACKAGE_SOURCE_DIR = $$PWD/android
