import QtQuick 2.0
import QtQuick.Controls 1.4

import "../style"

Item {
    id: root

    signal playClicked
    signal playbackSpeedSelected(var speed)

    property Item activeObject: null
    property bool revealed: false
    property bool running: false
    property alias snappingEnabled: snapCheckbox.checked

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

        Flickable {
            anchors.fill: parent
            Column {
                id: container
                anchors {
                    left: parent.left
                    right: parent.right
                    margins: 10
                }
                spacing: 10

                Loader {
                    anchors {
                        left: parent.left
                        right: parent.right
                    }

                    sourceComponent: (activeObject && activeObject.controls) ? activeObject.controls : playbackControls
                }

                CheckBox {
                    id: snapCheckbox
                    text: "Enable snapping"
                    checked: true
                }
            }
        }

        Component {
            id: playbackControls
            PlaybackControls {
                running: root.running
                onPlayClicked: {
                    root.playClicked()
                }
                onPlaybackSpeedSelected: {
                    root.playbackSpeedSelected(speed)
                }
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
