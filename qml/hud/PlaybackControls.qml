import QtQuick 2.0
import QtQuick.Layouts 1.1
import QtQuick.Controls 1.2

import "qrc:/qml/style"

Column {
    id: playbackControls

    signal playClicked
    signal playbackSpeedSelected(var speed)

    property bool running: false

    Text {
        text: "Playback controls"
    }

    Button {
        text: running ? "Pause" : "Play"
        onClicked: {
            playClicked()
        }
    }

    Text {
        text: "Playback speed"
    }

    ListView {
        id: playbackSpeedView
        anchors {
            left: parent.left
            right: parent.right
        }
        height: 100

        model: ListModel {
            id: playbackSpeedModel
            ListElement {
                key: "¼x"
                value: 0.25
            }
            ListElement {
                key: "½x"
                value: 0.5
            }
            ListElement {
                key: "1x"
                value: 1.0
            }
            ListElement {
                key: "2x"
                value: 2.0
            }
            ListElement {
                key: "4x"
                value: 4.0
            }
            ListElement {
                key: "8x"
                value: 8.0
            }
        }
        orientation: ListView.Horizontal
        clip: true
        preferredHighlightBegin: width / 2 - Style.touchableSize / 2
        preferredHighlightEnd: width / 2 + Style.touchableSize / 2
        delegate: Item {
            height: Style.touchableSize
            width: height
            Text {
                id: playbackSpeedText
                anchors.centerIn: parent
                text: model.key
            }
            MouseArea {
                anchors.fill: parent
                onClicked: {
                    playbackSpeedView.currentIndex = index
                }
            }
        }
        highlight: Rectangle {
            color: "white"
        }
        currentIndex: 2
        onCurrentIndexChanged: {
            if(!playbackSpeedModel) {
                return
            }
            playbackSpeedSelected(playbackSpeedModel.get(currentIndex).value)
        }
    }
}

