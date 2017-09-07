import QtQuick 2.0
import QtQuick.Controls 2.1
import QtQuick.Layouts 1.1
import QtQuick.Dialogs 1.0
import QtQuick.Window 2.1
import QtCharts 2.1
import QtMultimedia 5.5

import Neuronify 1.0

import "qrc:/qml/hud"

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
    height: Math.max(connectionText.height * 1.2, Style.touchableSize + 2 * Style.margin)
    color: "#f7fbff"
    border.color: "#9ecae1"
    border.width: 1.0
    
    Text {
        id: connectionText
        anchors {
            left: parent.left
            right: button.left
            margins: Style.margin
            verticalCenter: parent.verticalCenter
        }
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WrapAtWordBoundaryOrAnywhere
        font: Style.font
        text: "Touch other items to create a connection " +
              (fromThis ? "from" : "to") +
              " the selected item."
    }
    
    Rectangle {
        id: button
        anchors {
            top: parent.top
            bottom: parent.bottom
            right: parent.right
            margins: Style.margin
        }
        color: "#dee"
        width: Math.max(Style.touchableSize, doneText.width)
        height: Style.touchableSize
        Text {
            id: doneText
            anchors {
                verticalCenter: parent.verticalCenter
                horizontalCenter: parent.horizontalCenter
            }
            font: Style.font
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
