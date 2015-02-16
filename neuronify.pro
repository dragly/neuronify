TEMPLATE = app

TARGET = neuronify

android {
    TARGET = Neuronify
}

QT += qml quick widgets

HEADERS += \
    src/engine/neuronengine.h \
    src/io/fileio.h

SOURCES += \
    src/engine/neuronengine.cpp \
    src/io/fileio.cpp \
    src/main.cpp

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
