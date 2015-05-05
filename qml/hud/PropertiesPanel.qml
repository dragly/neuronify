import QtQuick 2.0

import "../style"

Item {
    id: root

    property Item activeObject: null
    property bool revealed: false

    anchors.fill: parent

    Rectangle {
        id: background
        anchors {
            right: parent.right
            top: parent.top
            rightMargin: -width
            bottom: parent.bottom
        }

        color: "#f7fbff"
        width: Style.device === "phone" ? parent.width * 0.5 : parent.width * 0.25

        border.color: "#9ecae1"
        border.width: 1.0

        MouseArea {
            anchors.fill: parent
        }

        Item {
            id: container
            anchors {
                fill: parent
                margins: 10
            }

            Loader {
                anchors.fill: parent
                sourceComponent: (activeObject && activeObject.controls) ? activeObject.controls : null
            }
        }

        states: State {
            when: root.revealed
            PropertyChanges {
                target: background
                anchors.rightMargin: 0.0
            }
        }

        transitions: Transition {
            NumberAnimation {
                target: background
                property: "anchors.rightMargin"
                duration: 400
                easing.type: Easing.InOutCubic
            }
        }
    }
}
