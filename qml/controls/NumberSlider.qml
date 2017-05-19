import QtQuick 2.7
import QtQuick.Controls 2.1
import QtQuick.Controls.Material 2.1
import QtQuick.Layouts 1.1

import Neuronify 1.0

Item {
    id: root
    property QtObject target: null
    property string property: ""
    property string name: "Placeholder"
    property real from: 0.0
    property real to: 1.0
    property var next: null
    property string unit: "m"
    property int precision: 2
    property real factor: 1

    property bool _ready: false

    Component.onCompleted: {
        _ready = true
    }

    implicitWidth: Math.max(label.implicitWidth, fieldMetrics.width)

    Binding {
        target: root.target
        property: root.property
        value: slider.value * factor
        when: _ready
    }

    Binding {
        target: slider
        property: "value"
        value: root.target[root.property] / factor
        when: _ready
    }
    
    Label {
        id: label
        property string unitString: (unit ? " (" + unit + ")" : "")
        anchors {
            top: parent.top
            horizontalCenter: parent.horizontalCenter
        }
        text: name + unitString
    }

    Slider {
        id: slider
        anchors {
            top: label.bottom
            bottom: field.top
            horizontalCenter: parent.horizontalCenter
        }
        orientation: Qt.Vertical
        from: root.from / factor
        to: root.to / factor
    }

    TextMetrics {
        id: fieldMetrics
        font: field.font
        text: "-00.00"
    }

    TextField {
        id: field
        anchors {
            bottom: parent.bottom
            left: parent.left
            right: parent.right
        }
        horizontalAlignment: Text.AlignHCenter
        KeyNavigation.tab: next

        onEditingFinished: {
            slider.value = Number(text)
        }
        Binding {
            target: field
            property: "text"
            value: slider.value.toFixed(precision)
        }
    }
}
