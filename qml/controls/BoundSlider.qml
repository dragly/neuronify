import QtQuick 2.0
import QtQuick.Controls 1.0

Column {
    id: root

    property QtObject target: null
    property string property: ""
    property string text: ""
    property string unit: ""
    property int precision: 1
    property real unitScale: 1.0

    property real minimumValue: slider.minimumValue
    property real maximumValue: slider.maximumValue
    property real stepSize: slider.stepSize

    width: parent.width

    Text {
        text: root.text ? (root.text + ": "
                           + slider.value.toFixed(precision) + " "
                           + root.unit) : ""
    }
    Slider {
        id: slider
        width: parent.width
        minimumValue: root.minimumValue / unitScale
        maximumValue: root.maximumValue / unitScale
    }
    Binding {
        target: root.target
        property: root.property
        value: slider.value * unitScale
    }
    Binding {
        target: slider
        property: "value"
        value: root.target[root.property] / unitScale
    }
}
