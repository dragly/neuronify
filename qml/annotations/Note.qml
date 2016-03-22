import QtQuick 2.0
import QtQuick.Controls 1.4
import QtQuick.Controls.Styles 1.4
import ".."
import "../style"

import Neuronify 1.0

Node {
    id: noteRoot
    objectName: "note"
    fileName: "annotations/Note.qml"
    square: true

    property alias text: textInput.text

    width: 180
    height: 120

    color: "#54B2FF"

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat(
                    ["text", "width", "height"])
    }

    onSelectedChanged: {
        transformMove.visible = !transformMove.visible
        if (!noteRoot.selected) {
             textInput.select(0, 0)
        }
    }

    Rectangle {
        anchors.fill: parent
        color: parent.color
    }

    TextArea {
        id: textInput
        anchors.fill: parent
        anchors.margins: 10
        horizontalAlignment: TextInput.AlignHCenter
        verticalAlignment: TextInput.AlignVCenter
        wrapMode: Text.WrapAtWordBoundaryOrAnywhere
        clip: true
        onFocusChanged: {
              noteRoot.selected = focus
        }

        visible: noteRoot.selected

        textFormat: Text.RichText

        style: TextAreaStyle {
            textColor: Style.font.color
            backgroundColor: "#AEDBFF"

         }
    }

    Text {
        anchors.fill: textInput.anchors.fill
        anchors.margins: textInput.anchors.margins
        text: textInput.text
        visible: !textInput.visible
        textFormat: textInput.textFormat
        horizontalAlignment: textInput.horizontalAlignment
        verticalAlignment: textInput.verticalAlignment
        wrapMode: textInput.wrapMode
        clip: textInput.clip
    }

    ResizeRectangle {
    }

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
        visible: false

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

