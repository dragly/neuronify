import QtQuick 2.0
import QtQuick.Controls 1.0

Column {
    id: root

    property QtObject target: null
    property string property: ""
    property string text: ""
    property string unit: ""
    property int precision: 1

    property alias minimumValue: slider.minimumValue
    property alias maximumValue: slider.maximumValue
    property alias stepSize: slider.stepSize

    width: parent.width

    Text {

        text: root.text ? (root.text + ": " + slider.value.toFixed(precision) + " " + root.unit) : ""
    }
    Slider {
        id: slider
        width: parent.width
        minimumValue: 0.0
        maximumValue: 5.0
    }
    Binding {
        target: root.target
        property: root.property
        value: slider.value
    }
    Binding {
        target: slider
        property: "value"
        value: root.target[root.property]
    }
}
