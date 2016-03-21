import QtQuick 2.0
import QtQuick.Controls 1.4
import QtQuick.Controls.Styles 1.4
import ".."
import "../style"

Node {
    id: noteRoot
    objectName: "note"
    fileName: "annotations/Note.qml"

    property alias text: textInput.text

    width: 180
    height: 120

    color: "#CCBD06"

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat(
                    ["text"])
    }

    Rectangle {
        anchors.fill: parent
        color: parent.color

        border.width: 1.0
        border.color: "#CCBD06"
    }

    TextArea {



        id: textInput
        anchors.fill: parent
        anchors.margins: 1
        horizontalAlignment: TextInput.AlignHCenter
        verticalAlignment: TextInput.AlignVCenter
        wrapMode: Text.WrapAtWordBoundaryOrAnywhere
        clip: true
        onFocusChanged: noteRoot.selected = focus

        textMargin: 10
        textFormat: Text.RichText

        style: TextAreaStyle {
            textColor: Style.font.color
            backgroundColor: "#FFF144"

         }




    }

    ResizeRectangle {
    }

    Rectangle {
        anchors {
            horizontalCenter: parent.left
            verticalCenter: parent.top
        }
        width: parent.height / 3
        height: width
        radius: width / 2
        color: "#c6dbef"
        border.width: width * 0.1
        border.color: "#f7fbff"

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

