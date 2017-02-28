import QtQuick 2.0
import QtQuick.Layouts 1.1
import QtQuick.Controls 1.2

import "qrc:/qml"
import "qrc:/qml/style"

Rectangle {
    id: playbackRoot

    property alias autoHideEnabled: autoHideTimer.running
    property real playbackSpeed: 1.0
    property bool revealed

    function revealTemporarily() {
        if(!revealed) {
            revealed = true
            autoHideTimer.restart()
        }
    }

    function revealPermanently() {
        autoHideTimer.stop()
        revealed = true
    }

    function toggleRevealPermanently() {
        autoHideTimer.stop()
        revealed = !revealed
    }

    function toggleSpeed(eventKey) {
        switch(eventKey) {
        case Qt.Key_0:
            playbackControls.currentIndex = 0
            revealTemporarily()
            break
        case Qt.Key_1:
            playbackControls.currentIndex = 1
            revealTemporarily()
            break
        case Qt.Key_2:
            playbackControls.currentIndex = 2
            revealTemporarily()
            break
        case Qt.Key_3:
            playbackControls.currentIndex = 3
            revealTemporarily()
            break
        case Qt.Key_4:
            playbackControls.currentIndex = 4
            revealTemporarily()
            break
        }
    }

    anchors {
        bottom: parent.bottom
        bottomMargin: -height
        horizontalCenter: parent.horizontalCenter
        margins: Style.touchableSize * 0.25
    }

    radius: height * 0.5

    color: Style.color.background
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

        onCurrentIndexChanged: {
            playbackSpeed = playbackSpeedModel.get(playbackControls.currentIndex).value
        }

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
                    asynchronous: true
                }
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        playbackControls.currentIndex = index
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

    MouseArea {
        anchors.fill: parent
        enabled: autoHideTimer.running
        propagateComposedEvents: true
        onClicked: {
            revealPermanently()
            mouse.accepted = false
        }
    }

    Timer {
        id: autoHideTimer
        interval: 2000
        onTriggered: {
            revealed = false
        }
    }
}
