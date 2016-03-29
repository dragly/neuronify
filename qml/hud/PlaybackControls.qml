import QtQuick 2.0
import QtQuick.Layouts 1.1
import QtQuick.Controls 1.2

import "qrc:/qml/style"

Rectangle {
    id: playbackRoot

    property real playbackSpeed: 1.0

    anchors {
        bottom: parent.bottom
        horizontalCenter: parent.horizontalCenter
        margins: Style.touchableSize * 0.25
    }

    radius: height * 0.4

    color: Style.color.background
    border.width: Style.border.width
    border.color: Style.color.border
    width: playbackControls.count * playbackControls.height + playbackControls.anchors.margins * 2
    height: Style.touchableSize * 1.5

    ListView {
        id: playbackControls

        anchors {
            fill: parent
            margins: Style.touchableSize * 0.25
        }

        interactive: false
        orientation: ListView.Horizontal
        model: ListModel {
            id: playbackSpeedModel
            ListElement {
                image: "qrc:/images/playback/pause.svg"
                value: 0.0
            }
            ListElement {
                image: "qrc:/images/playback/play.svg"
                value: 1.0
            }
            ListElement {
                image: "qrc:/images/playback/fast.svg"
                value: 2.0
            }
            ListElement {
                image: "qrc:/images/playback/superfast.svg"
                value: 4.0
            }
            ListElement {
                image: "qrc:/images/playback/superduperfast.svg"
                value: 8.0
            }
        }
        delegate: Item {
            height: playbackControls.height
            width: height
            Image {
                anchors {
                    fill: parent
                    margins: parent.width * 0.05
                }
                source: model.image
            }
            MouseArea {
                anchors.fill: parent
                onClicked: {
                    playbackControls.currentIndex = index
                    playbackSpeed = playbackSpeedModel.get(playbackControls.currentIndex).value
                }
            }
        }
        highlight: Rectangle {
            color: Style.color.background
            border.width: Style.border.width
            border.color: Style.color.border
            radius: width * 0.5
        }
        Binding {
            target: playbackControls
            property: "currentIndex"
            value: {
                for(var i = 0; i < playbackSpeedModel.count; i++) {
                    if(playbackSpeedModel.get(i).value === playbackSpeed) {
                        return i;
                    }
                }
                return 0;
            }
        }
    }
}
