import QtQuick 2.6
import QtQuick.Controls 2.1

import Neuronify 1.0

import "qrc:/"
import "qrc:/qml/"
import "qrc:/qml/controls"
import "qrc:/qml/style"

Item {
    id: root
    property string text: ""
    property string info: ""
    default property alias components: propertiesPage.children
    property StackView stackView: parent && parent.stackView ? parent.stackView : null

    width: parent.width
    height: Style.control.fontMetrics.height * 2.8

    PropertiesPage {
        id: propertiesPage
        visible: false
        title: root.text
    }

    Rectangle {
        anchors.fill: parent
        color: "#11000000"
        visible: mouseArea.pressed
    }

    Rectangle {
        anchors {
            left: parent.left
            right: parent.right
            bottom: parent.bottom
        }
        height: Style.border.width
        color: Style.border.lightColor
    }

    Column {
        anchors {
            left: parent.left
            verticalCenter: parent.verticalCenter
        }
        spacing: buttonText.height * 0.1

        Text {
            id: buttonText
            text: root.text
            font: Style.control.font
            color: Style.control.text.color
        }
        Text {
            id: subText
            font: Style.control.subText.font
            color: Style.control.subText.color
            textFormat: Text.RichText
            text: root.info
        }
    }

    Image {
        anchors {
            right: parent.right
            top: parent.top
            bottom: parent.bottom
            margins: parent.height * 0.1
        }
        width: height
        fillMode: Image.PreserveAspectFit
        source: "qrc:/images/tools/back.png"
        rotation: 180
    }
    MouseArea {
        id: mouseArea
        anchors.fill: parent
        onClicked: {
            stackView.push(propertiesPage)
        }
    }
}
