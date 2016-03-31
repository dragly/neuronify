import QtQuick 2.0
import QtQuick.Layouts 1.1
import QtQuick.Controls 1.2

import "qrc:/qml/style"

Rectangle {
    id: playbackRoot

    property real playbackSpeed: 1.0
    property bool revealed

    anchors {
        bottom: parent.bottom
        bottomMargin: -height
        horizontalCenter: parent.horizontalCenter
        margins: Style.touchableSize * 0.25
    }

    radius: height * 0.5

    color: Style.color.background
//    border.width: Style.border.width
//    border.color: Style.border.lightColor
    width: playbackControls.width + Style.touchableSize * 0.3
    height: playbackControls.height + Style.touchableSize * 0.3

    state: revealed ? "revealed" : "hidden"

    Row {
        id: playbackControls
        property int currentIndex: 0
        anchors {
            centerIn: parent
        }
        height: Style.touchableSize
        Repeater {
            anchors {
                fill: parent
                margins: Style.touchableSize * 0.25
            }
            model: ListModel {
                id: playbackSpeedModel
                ListElement {
                    image: "pause"
                    value: 0.0
                }
                ListElement {
                    image: "play"
                    value: 1.0
                }
                ListElement {
                    image: "fast"
                    value: 2.0
                }
                ListElement {
                    image: "superfast"
                    value: 4.0
                }
                ListElement {
                    image: "superduperfast"
                    value: 8.0
                }
            }
            Item {
                height: Style.touchableSize
                width: height
                Image {
                    property string active: playbackControls.currentIndex === index ? "-active" : ""
                    anchors {
                        fill: parent
                        margins: parent.width * 0.05
                    }
                    source: "qrc:/images/playback/" + model.image + active + ".png"
                }
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        playbackControls.currentIndex = index
                        playbackSpeed = playbackSpeedModel.get(playbackControls.currentIndex).value
                    }
                }
            }
        }

        Item {
            height: Style.touchableSize
            width: height
            Image {
                id: hideImage
                anchors {
                    fill: parent
                    margins: parent.width * 0.2
                }
                width: Style.touchableSize
                height: width
                source: "qrc:/images/back.png"
                rotation: 270
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        playbackRoot.revealed = false
                    }
                }
            }
        }
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

    states: [
        State {
            name: "revealed"
            PropertyChanges {
                target: playbackRoot
                anchors.bottomMargin: playbackRoot.anchors.margins
            }
        },
        State {
            name: "hidden"
        }
    ]

    transitions: [
        Transition {
            NumberAnimation {
                property: "anchors.bottomMargin"
                duration: 400
                easing.type: Easing.InOutQuad
            }
        }
    ]
}
