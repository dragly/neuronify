import QtQuick 2.0
import QtQuick.Window 2.0
import QtQuick.Layouts 1.3
import Qt.labs.folderlistmodel 2.1

import "../.."
import "../style"

Rectangle {
    id: iconRoot
    signal clicked
    property url basePath
    property url filePath: basePath + ".nfy"
    property url imagePath: basePath + ".png"
    property bool empty: false

    Layout.fillWidth : true
    Layout.fillHeight: true
    width : 1
    height : 1
    color: Style.color.background
    border.color: Style.border.color
    border.width: Style.border.width

    Image{
        id: iconImage

        asynchronous: true
        anchors.fill: parent

        smooth: true
        antialiasing: true

        source: empty ? "" : imagePath
        fillMode: Image.PreserveAspectCrop
    }

    Text {
        id: saveText
        anchors.centerIn: parent
        text: iconRoot.empty ? "Empty file" : ""
        font: Style.button.font
        renderType: Text.QtRendering
        color: Style.button.color
    }

    MouseArea{
        anchors.fill: parent
        onClicked: {
            iconRoot.clicked()
        }
    }
}

