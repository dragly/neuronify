import QtQuick 2.0

import "../style"

Item {
    id: root

    signal clicked

//    anchors {
//        right: parent.right
//        top: parent.top
//    }
    width: Style.touchableSize * 1.5
    height: width
    Image {
        width: Style.touchableSize
        height: width
        anchors.centerIn: parent

        source: "qrc:/images/tools/create.png"
    }

    MouseArea {
        anchors.fill: parent
        onClicked: {
            root.clicked()
        }
    }
}

