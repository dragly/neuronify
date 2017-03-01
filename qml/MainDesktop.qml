import QtQuick 2.5
import QtQuick.Controls 1.4
import QtQuick.Dialogs 1.0
import QtQuick.Layouts 1.1
import QtQuick.Particles 2.0
import QtQuick.Window 2.1

import QtCharts 2.1
import QtMultimedia 5.5
import Qt.labs.settings 1.0

import Neuronify 1.0
import CuteVersioning 1.0
import QtGraphicalEffects 1.0

import "qrc:/qml/hud"
import "qrc:/qml/menus/mainmenu"
import "qrc:/qml/style"
import "qrc:/qml/io"
import "qrc:/qml/tools"
import "qrc:/qml/controls"

Item {
    id: root
    Neuronify {
        id: neuronify
        anchors {
            left: parent.left
            right: parent.right
            top: parent.top
            bottom: parent.bottom
        }
        clip: true
    }

    Rectangle {
        id: leftMenu
        anchors {
            left: parent.left
            top: parent.top
            bottom: parent.bottom
            leftMargin: 0
        }

        width: 128

        color: "#6BAED6"
        z: 14

//        Rectangle {
//            anchors {
//                left: parent.left
//                right: parent.right
//            }
//            y: 480
//            height: 64

//            color: itemMenu.color
//        }

        Image {
            id: logo
            anchors {
                left: parent.left
                right: parent.right
                top: parent.top
                margins: parent.width * 0.2
                topMargin: 48
            }
            fillMode: Image.PreserveAspectFit
            height: width
            source: "qrc:/images/logo/logo-no-background.png"
            mipmap: true
        }

        Text {
            id: logoText
            color: "white"
            font.pixelSize: 24
            font.family: Style.font.family
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WrapAtWordBoundaryOrAnywhere
            text: "Neuronify\n" + Version.latestTag
        }

        ShaderEffectSource {
            id: logoTextCopy
            anchors {
                left: parent.left
                right: parent.right
                top: logo.bottom
//                top: parent.top
                margins: 8
            }
            height: width * logoText.height / logoText.width
            hideSource: true
            sourceItem: logoText
            smooth: true
            antialiasing: true
        }

        MouseArea {
            anchors.fill: parent
            onClicked: {
                root.state = "creation"
            }
        }
    }

    HudShadow {
        anchors.fill: leftMenu
        source: leftMenu
        z: 13
    }

    Rectangle {
        id: itemMenu
        anchors {
            left: leftMenu.right
//            left: parent.left
            leftMargin: -width - 32
            top: parent.top
//            topMargin: 160
            bottom: parent.bottom
//            bottomMargin: 120
        }

        width: 160 + 32
        radius: 0

        color: "#e3eef9"

        MouseArea {
            anchors.fill: parent
            onClicked: root.state = ""
        }
        z: 11
    }

    HudShadow {
        anchors.fill: itemMenu
        source: itemMenu
        z: 12
    }

    Rectangle {
        id: subItemMenu
        anchors {
            left: itemMenu.right
            leftMargin: -width - 32
            top: itemMenu.top
            bottom: itemMenu.bottom
//            topMargin: 64
        }
        color: "white"
        width: 160
        height: 480
        z: 10
        state: itemMenu.state
    }

    HudShadow {
        anchors.fill: subItemMenu
        source: subItemMenu
    }

    states: [
        State {
            name: "creation"
            PropertyChanges { target: itemMenu; anchors.leftMargin: -32 }
            PropertyChanges { target: subItemMenu; anchors.leftMargin: -32}
            PropertyChanges { target: leftMenu; width: 72 }
            PropertyChanges { target: logoTextCopy; opacity: 0.0 }
        }
    ]

    transitions: [
        Transition {
            NumberAnimation {
                targets: [itemMenu]
                properties: "anchors.leftMargin"
                duration: 600
                easing.type: Easing.InOutQuad
            }
            NumberAnimation {
                targets: [subItemMenu]
                properties: "anchors.leftMargin"
                duration: 800
                easing.type: Easing.InOutQuad
            }
            NumberAnimation {
                target: leftMenu
                property: "width"
                duration: 400
                easing.type: Easing.InOutQuad
            }
            NumberAnimation {
                target: logoTextCopy
                property: "opacity"
                duration: 400
                easing.type: Easing.InOutQuad
            }
        }
    ]
}
