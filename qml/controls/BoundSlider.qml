import QtQuick 2.0
import QtQuick.Controls 1.0

Slider {
    id: root

    property QtObject target: null
    property string property: ""

    width: parent.width
    minimumValue: 0.0
    maximumValue: 5.0
    Binding {
        target: root.target
        property: root.property
        value: root.value
    }
    Binding {
        target: root
        property: "value"
        value: root.target[root.property]
    }
}

