import QtQuick 2.6
import QtQuick.Controls 1.4
import QtQuick.Controls.Styles 1.4
import QtQml 2.2
import QtGraphicalEffects 1.0

import "qrc:/qml/style"

/*!
\qmltype SwitchControl
\inqmlmodule Neuronify
\ingroup neuronify-controls
\brief Switch control
*/


Item {
    id: root

    property QtObject target: null
    property string property: ""
    property alias checked: switchRoot.checked
    property string checkedText: "checked"
    property string uncheckedText: "unchecked"

    width: parent.width
    height: Style.control.fontMetrics.height * 2

    Text {
        anchors {
            left: parent.left
            right: switchRoot.left
            verticalCenter: parent.verticalCenter
        }
        text: switchRoot.checked ? checkedText : uncheckedText
    }

    Switch{
        id: switchRoot

        width: parent.width * 0.2
        height: parent.height * 0.8

        checked: root.target[root.property]
        anchors {
            right: parent.right
            verticalCenter: parent.verticalCenter
        }

        style: SwitchStyle {
            groove: Rectangle {
                implicitWidth: switchRoot.width
                implicitHeight: switchRoot.height
                radius: height/2
                color: Style.color.background
                border.color: "#9ecae1"
                border.width: Style.border.width
            }
            handle: Rectangle {
                implicitWidth:  switchRoot.width * 0.5
                implicitHeight: switchRoot.height
                radius: height/2
                color: "#9ecae1"
                border.color: "#9ecae1"
                border.width: Style.border.width
                gradient: grad
            }
        }

    }

    Gradient {
        id: grad
        GradientStop { position: 0.0; color: Style.color.foreground }
        GradientStop { position: 1.0; color: Style.color.border }
    }

    Binding {
        target: root.target
        property: root.property
        value: switchRoot.checked
    }
}
