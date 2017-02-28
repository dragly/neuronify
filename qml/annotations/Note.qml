import QtQuick 2.0
import QtQuick.Controls 1.4
import QtQuick.Controls.Styles 1.4
import ".."
import "../controls"
import "../style"

import Neuronify 1.0

/*!
\qmltype Note
\inqmlmodule Neuronify
\ingroup neuronify-annotations
\brief Annotate simulations with notes
*/


Node {
    id: noteRoot

    property string text: "Open properties to change this text"

    objectName: "note"
    filename: "annotations/Note.qml"
    square: true
    snapToCenter: false

    canReceiveConnections: false

    width: 180
    height: 120

    color: "white"
    savedProperties: PropertyGroup {
        property alias text: noteRoot.text
        property alias width: noteRoot.width
        property alias height: noteRoot.height
    }

    controls: Component {
        PropertiesPage {
            property string title: "Note"
            spacing: Style.spacing
            Text {
                text: "Text:"
                font: Style.control.font
                color: Style.text.color
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
        id: background
        anchors.fill: parent
        color: parent.color
//        border.color: Style.meter.border.color
//        border.width: Style.meter.border.width
        antialiasing: true
    }

    ItemShadow {
        source: background
        anchors.fill: background
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
}

