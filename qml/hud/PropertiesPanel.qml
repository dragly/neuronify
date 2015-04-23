import QtQuick 2.0

import "../style"

Item {
    id: root

    default property alias rectChildren: rect.children
    property bool revealed: false

    anchors.fill: parent

    MouseArea {
        anchors.fill: parent
        enabled: revealed
        propagateComposedEvents: true
        onPressed: {
            root.revealed = false
            mouse.accepted = false
        }
    }

    Rectangle {
        id: rect
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

        states: State {
            when: root.revealed
            PropertyChanges {
                target: rect
                anchors.rightMargin: 0.0
            }
        }

        transitions: Transition {
            NumberAnimation {
                target: rect
                property: "anchors.rightMargin"
                duration: 400
                easing.type: Easing.InOutCubic
            }
        }

        MouseArea {
            anchors.fill: parent
        }
    }

}
