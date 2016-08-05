import QtQuick 2.0
import "../../style"

Item {
    id: buttonRoot
    signal clicked
    property alias text: buttonText.text
    property real maximumRotation: Math.random() * 5 + 5

    height: Style.touchableSize * 0.9

    smooth: true

    Rectangle {
        anchors.fill: parent
        color: Style.button.backgroundColor
        antialiasing: true
    }

    Text {
        id: buttonText
        anchors {
            left: parent.left
            leftMargin: Style.touchableSize
            verticalCenter: parent.verticalCenter
        }

        text: "Begin"
        font: Style.button.font
        renderType: Text.QtRendering
        color: Style.button.color
    }

    MouseArea {
        anchors.fill: parent
        onClicked: {
            buttonRoot.clicked()
        }
    }
}
