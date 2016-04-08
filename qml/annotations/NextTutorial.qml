import QtQuick 2.0

import Neuronify 1.0

import ".."
import "../style"
import "../tools"

Node {
    id: root

    property string targetSimulation: ""

    filename: "annotations/NextTutorial.qml"
    objectName: "NextTutorial"
    color: "#54B2FF"

    width: 196
    height: width

    savedProperties: PropertyGroup {
        property alias targetSimulation: root.targetSimulation
    }

    Rectangle {
        id: background

        anchors.fill: parent
        color: root.color
        border.color: Style.border.color
        border.width: Style.border.width
        radius: width * 0.2

        Column {
            id: column
            anchors.centerIn: parent
            width: parent.width
            Text {
                horizontalAlignment: Text.AlignHCenter
                anchors {
                    left: parent.left
                    right: parent.right
                    margins: background.radius * 0.25
                }
                font.pixelSize: 24
                wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                text: "Next tutorial"
                color: "white"
            }
            Image {
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width * 0.5
                height: width
                fillMode: Image.PreserveAspectFit
                source: "qrc:/images/annotate/next.png"
            }
        }

        MouseArea {
            anchors.fill: column
            onClicked: {
                simulator.loadSimulation(targetSimulation)
            }
        }
    }
}
