HEADERS += $$PWD/QmlPreviewer

ios|android|winrt {
    HEADERS += $$PWD/qmlpreviewer_stub.h
} else {
    HEADERS += $$PWD/qmlpreviewer.h
    SOURCES += $$PWD/qmlpreviewer.cpp
}

INCLUDEPATH += $$PWD
RESOURCES += \
    $$PWD/qmlpreviewer.qrc

DISTFILES += \
    $$PWD/qtquickcontrols2.conf

