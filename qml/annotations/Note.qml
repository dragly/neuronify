import QtQuick 2.0
import QtQuick.Controls 1.4
import QtQuick.Controls.Styles 1.4
import ".."
import "../style"

import Neuronify 1.0

Node {
    id: noteRoot

    property string text: "Open properties to change this text"

    objectName: "note"
    fileName: "annotations/Note.qml"
    square: true

    canReceiveConnections: false

    width: 180
    height: 120

    color: "#54B2FF"
    savedProperties: PropertyGroup {
        property alias text: noteRoot.text
        property alias width: noteRoot.width
        property alias height: noteRoot.height
    }

    controls: Component {
        Column {
            spacing: Style.spacing
            Text {
                text: "Text:"
            }
            TextArea {
                id: textInput
                anchors {
                    left: parent.left
                    right: parent.right
                }
                text: noteRoot.text
                height: 200
                wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                clip: true
                textFormat: Text.PlainText
                onFocusChanged: {
                    noteRoot.selected = focus
                }
                Binding {
                    target: noteRoot
                    property: "text"
                    value: textInput.text
                }
                Binding {
                    target: textInput
                    property: "text"
                    value: noteRoot.text
                }
            }
        }
    }

    Rectangle {
        anchors.fill: parent
        color: parent.color
    }

    Text {
        anchors.fill: parent
        anchors.margins: 10
        text: noteRoot.text
        textFormat: Text.PlainText
        horizontalAlignment: TextInput.AlignHCenter
        verticalAlignment: TextInput.AlignVCenter
        wrapMode: Text.WrapAtWordBoundaryOrAnywhere
        clip: true
        font.pixelSize: 20
    }

    ResizeRectangle {}

    Rectangle {
        id: transformMove

        anchors {
            horizontalCenter: parent.left
            verticalCenter: parent.top
        }
        width: 32
        height: width
        radius: width / 2
        color: "#c6dbef"
        border.width: width * 0.1
        border.color: "#f7fbff"
        visible: noteRoot.selected

        Image {

            anchors.fill: parent
            anchors.margins: parent.width * 0.1
            source: "qrc:/images/transform-move.png"
            smooth: true
            antialiasing: true
        }

        MouseArea {
            anchors.fill: parent
            drag.target: noteRoot
            onPressed: {
                noteRoot.dragging = true
                dragStarted()
            }

            onClicked: {
                noteRoot.clicked(noteRoot, mouse)
            }

            onReleased: {
                noteRoot.dragging = false
            }
        }
    }
}

