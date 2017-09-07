import QtQuick 2.5

import "qrc:/qml/hud"

import "qrc:/qml/style"

Item {
    id: root
    signal clicked
    property alias source: image.source
    width: Style.touchableSize * 1.5
    height: width
    Image {
        id: image
        width: Style.touchableSize
        height: width
        anchors.centerIn: parent
    }
    
    MouseArea {
        anchors.fill: parent
        onClicked: {
            root.clicked()
        }
    }
}
