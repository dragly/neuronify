import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Dialogs 1.0
import QtQuick.Window 2.1
import QtCharts 2.1
import QtMultimedia 5.5

import Neuronify 1.0

import "qrc:/qml/hud"
import "qrc:/qml/menus/mainmenu"
import "qrc:/qml/style"
import "qrc:/qml/io"
import "qrc:/qml/tools"

Rectangle {

    signal doneClicked
    property bool fromThis: false

    anchors {
        top: parent.top
        left: parent.left
        right: parent.right
    }
    height: Style.touchableSize * 1.5
    color: "yellow"
    
    Text {
        anchors.centerIn: parent
        horizontalAlignment: Text.AlignHCenter
        font: Style.control.font
        text: "Connection mode\nClicking other items connects them\n" +
              (fromThis ? "from" : "to") +
              " the currently selected."
    }
    
    Rectangle {
        anchors {
            verticalCenter: parent.verticalCenter
            right: parent.right
            rightMargin: Style.margin
        }
        width: Math.max(Style.touchableSize, doneText.width)
        height: Style.touchableSize
        Text {
            id: doneText
            anchors {
                verticalCenter: parent.verticalCenter
                horizontalCenter: parent.horizontalCenter
            }
            font: Style.control.font
            text: "Done"
        }
        MouseArea {
            anchors.fill: parent
            onClicked: {
                doneClicked()
            }
        }
    }
}
