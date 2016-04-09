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
    property bool isChecked: switchRoot.checked
    property string checkedText: "checked"
    property string uncheckedText: "unchecked"

    width: parent.width
    height: Style.control.fontMetrics.height * 2

    Switch{
        id: switchRoot
        checked: root.target[root.property]

        style: SwitchStyle {
            groove: Rectangle {
                Text{
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.leftMargin:parent.radius
                    text: checkedText
                }
                Text{
                    anchors{
                        right: parent.right
                        verticalCenter: parent.verticalCenter
                        rightMargin: parent.radius
                    }
                    text: uncheckedText
                }
                implicitWidth: root.width
                implicitHeight: root.height
                radius: height/2
                color: Style.color.background
                border.color: "#9ecae1"
                border.width: Style.border.width
            }
            handle: Rectangle {
                implicitWidth:  root.width * 0.5
                implicitHeight: root.height
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
