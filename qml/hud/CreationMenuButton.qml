import QtQuick 2.0

import "../style"

Image {
    id: root

    signal clicked

    anchors {
        right: parent.right
        top: parent.top
        margins: Style.margin
    }
    width: Style.touchableSize
    height: width

    source: "qrc:/images/back.png"

    MouseArea {
        anchors.fill: parent
        onClicked: {
            root.clicked()
        }
    }
}

