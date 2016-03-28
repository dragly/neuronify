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

    anchors {
        top: parent.top
        left: parent.left
        right: parent.right
    }
    height: Style.touchableSize
    
    Text {
        anchors.centerIn: parent
        text: "Connection mode: Select other items to connect them."
    }
    
    Button {
        anchors {
            verticalCenter: parent.verticalCenter
            right: parent.right
        }
        text: "Done"
        onClicked: {
            doneClicked()
        }
    }
}
