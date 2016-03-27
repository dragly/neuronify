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

    property real minimumValue: -1.0
    property real maximumValue: 1.0
    property real stepSize: 0.01

    width: parent.width

    Text {
        text: root.text ? (root.text + ": " + slider.value.toFixed(precision) + " " + root.unit) : ""
    }
    Slider {
        id: slider
        width: parent.width
        minimumValue: root.minimumValue / unitScale
        maximumValue: root.maximumValue / unitScale
        stepSize: root.stepSize / unitScale
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
