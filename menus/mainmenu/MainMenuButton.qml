import QtQuick 2.0
import "../../style"

Item {
    id: buttonRoot
    signal clicked
    property alias text: buttonText.text
    property real maximumRotation: Math.random() * 5 + 5

    smooth: true

    transform: Rotation { id: rotationTransform
        origin.x: 30
        origin.y: 30
        axis {
            x: 0
            y: 1
            z: 0
        }
        angle: 0
    }

    Rectangle {
        anchors.fill: parent
        color: Style.button.color
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
        font.pixelSize: Style.button.fontSize
        font.weight: Font.Light
        font.family: "Roboto"
        renderType: Text.QtRendering
        color: Style.button.fontColor
    }

    MouseArea {
        anchors.fill: parent
        onClicked: {
            buttonRoot.clicked()
        }
    }
}
