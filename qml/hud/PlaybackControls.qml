import QtQuick 2.0
import QtQuick.Layouts 1.1
import QtQuick.Controls 1.2

import "qrc:/qml/style"

Column {
    id: playbackControls

    signal playClicked
    signal playbackSpeedSelected(var speed)

    property var workspace
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

    Binding {
        target: playbackSpeedView
        property: "currentIndex"
        value: {
            if(!workspace) {
                return 0;
            }
            for(var i = 0; i < playbackSpeedModel.count; i++) {
                if(playbackSpeedModel.get(i).value === workspace.playbackSpeed) {
                    return i;
                }
            }
            return 0;
        }
    }

    GridView {
        id: playbackSpeedView
        anchors {
            left: parent.left
            right: parent.right
        }
        height: Style.touchableSize * 3.0
        cellWidth: Style.touchableSize
        cellHeight: Style.touchableSize
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
        clip: true
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
            color: "#deebf7"
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

