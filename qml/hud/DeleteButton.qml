import QtQuick 2.0

import ".."
import "../style"

Item {
    id: root

    signal clicked

    property bool revealed: false

    anchors {
        left: parent.left
        bottom: parent.bottom
        leftMargin: -width
    }

    width: Style.touchableSize * 2
    height: width
    Image {
        width: Style.touchableSize
        height: width
        anchors.centerIn: parent

        fillMode: Image.PreserveAspectFit

        source: "../../images/delete.png"
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
            anchors.leftMargin: 0
        }
    }

    transitions: Transition {
        NumberAnimation {
            property: "anchors.leftMargin"
            duration: 400
            easing.type: Easing.InOutQuad
        }
    }
}

