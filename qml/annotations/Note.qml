import QtQuick 2.0
import QtQuick.Controls 1.2
import ".."

Node {
    id: noteRoot
    objectName: "note"
    fileName: "annotations/Note.qml"

    property alias text: textInput.text

    width: 180
    height: 120

    color: "#F7EE72"

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat(
                    ["text"])
    }

    onSelectedChanged: {
        textInput.focus = selected
    }

    Rectangle {
        anchors.fill: parent
        color: parent.color
    }

    TextInput {
        id: textInput
        anchors.fill: parent
        anchors.margins: 10
        horizontalAlignment: TextInput.AlignHCenter
        verticalAlignment: TextInput.AlignVCenter
        wrapMode: Text.WrapAtWordBoundaryOrAnywhere
        clip: true
        onFocusChanged: noteRoot.selected = focus
    }

    ResizeRectangle {
    }

    Rectangle {
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
        visible: textInput.activeFocus

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

