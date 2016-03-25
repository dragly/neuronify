import QtQuick 2.0

import "../style"

Item {
    id: root

    signal clicked

    anchors {
        right: parent.right
        top: parent.top
    }
    width: Style.touchableSize * 2
    height: width
    Image {
        width: Style.touchableSize
        height: width
        anchors.centerIn: parent

        source: "qrc:/images/back.png"
    }

    MouseArea {
        anchors.fill: parent
        onClicked: {
            root.clicked()
        }
    }
}

