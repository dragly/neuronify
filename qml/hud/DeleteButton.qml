import QtQuick 2.0

import ".."
import "../style"

Item {
    id: root

    signal clicked

    property bool revealed: false

    opacity: 0

    width: Style.touchableSize * 1.5
    height: width
    Image {
        width: Style.touchableSize
        height: width
        anchors.centerIn: parent

        fillMode: Image.PreserveAspectFit

        source: "qrc:/images/tools/delete.png"
        antialiasing: true
        smooth: true
    }

    MouseArea {
        anchors.fill: parent
        onClicked: {
            root.clicked()
        }
    }

    states: State {
        when: revealed
        PropertyChanges {
            target: root
//            anchors.rightMargin: 0
            opacity: 1.0
        }
    }

    transitions: Transition {
        NumberAnimation {
            property: "opacity"
            duration: 400
            easing.type: Easing.InOutQuad
        }
    }
}

