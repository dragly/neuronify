import QtQuick 2.5
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Dialogs 1.0
import QtQuick.Window 2.1
import QtCharts 2.1
import QtMultimedia 5.5
import Qt.labs.settings 1.0

import Neuronify 1.0

import "qrc:/qml/hud"
import "qrc:/qml/menus/mainmenu"
import "qrc:/qml/style"
import "qrc:/qml/io"
import "qrc:/qml/tools"
import "qrc:/qml/controls"

Item {
    id: playbackButton
    
    signal clicked
    
    property bool revealed: true
    
    anchors {
        horizontalCenter: parent.horizontalCenter
        bottom: parent.bottom
    }
    width: Style.touchableSize * 2
    height: width
    
    enabled: revealed
    state: revealed ? "revealed" : "hidden"
    
    states: [
        State {
            name: "hidden"
            PropertyChanges {
                target: playbackButton
                opacity: 0.0
            }
        },
        State {
            name: "revealed"
            PropertyChanges {
                target: playbackButton
                opacity: 1.0
            }
        }
    ]
    
    transitions: [
        Transition {
            NumberAnimation {
                properties: "opacity"
                duration: 200
                easing.type: Easing.InOutQuad
            }
        }
    ]
    Image {
        
        anchors.centerIn: parent
        width: Style.touchableSize
        height: width
        
        source: "qrc:/images/playback/playpause.png"
    }
    
    
    MouseArea {
        anchors.fill: parent
        onClicked: {
            playbackButton.clicked()
        }
    }
}
