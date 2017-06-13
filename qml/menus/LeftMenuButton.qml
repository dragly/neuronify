import QtQuick 2.5
import QtQuick.Controls 2.1
import QtQuick.Controls.Material 2.1
import QtQuick.Dialogs 1.0
import QtQuick.Layouts 1.3
import QtQuick.Particles 2.0
import QtQuick.Window 2.1

import QtCharts 2.1
import QtMultimedia 5.5
import Qt.labs.settings 1.0
import Qt.labs.folderlistmodel 2.1
import Qt.labs.platform 1.0

import Neuronify 1.0
import CuteVersioning 1.0
import QtGraphicalEffects 1.0

import "qrc:/qml/backend"
import "qrc:/qml/controls"
import "qrc:/qml/hud"
import "qrc:/qml/io"
import "qrc:/qml/menus/filemenu"
import "qrc:/qml/menus/mainmenu"
import "qrc:/qml/tools"
import "qrc:/qml/store"
import "qrc:/qml/style"
import "qrc:/qml/ui"

MouseArea {
    id: root
    property alias text: textItem.text
    property alias icon: icon_
    property alias duration: transtionAnimation.duration
    property color color: "white"

    Layout.fillHeight: true
    Layout.fillWidth: true
    Layout.minimumWidth: 48
    Layout.maximumWidth: parent.height
    state: parent.containsMouse ? "" : "discrete"

    Item {
        id: menuItemColumn
        anchors.fill: parent
        Item {
            id: imageContainer
            anchors {
                top: parent.top
                horizontalCenter: parent.horizontalCenter
            }
            width: 0.6 * parent.width
            height: 0.6 * parent.height
            MaterialIcon {
                id: icon_
                anchors.fill: parent
                color: root.color
                category: "content"
                name: "create"
            }
        }
        Item {
            anchors {
                top: imageContainer.bottom
                topMargin: 6
                left: parent.left
                right: parent.right
                bottom: parent.bottom
            }

            Text {
                id: textItem
                anchors.fill: parent
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignTop
                color: root.color
                fontSizeMode: Text.Fit
                minimumPixelSize: 4
                font.pixelSize: 11

                text: "icon"
            }
        }
    }

    states: [
        State {
            name: "discrete"
            PropertyChanges {
                target: imageContainer
                height: parent.height
            }
            PropertyChanges {
                target: textItem
                opacity: 0.0
            }
        }
    ]
    transitions: [
        Transition {
            from: "discrete"
            reversible: true
            SequentialAnimation {
                PauseAnimation {
                    duration: 200
                }
                NumberAnimation {
                    id: transtionAnimation
                    properties: "height,opacity"
                    duration: 240
                    easing.type: Easing.InOutQuad
                }
            }
        }

    ]
}
