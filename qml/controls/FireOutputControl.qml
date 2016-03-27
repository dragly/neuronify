import QtQuick 2.0
import QtQuick.Controls 1.0

Column {
    id: root
    property QtObject target: null
    property string property: "fireOutput"
    property alias minimumValue: fireOutputSlider.minimumValue
    property alias maximumValue: fireOutputSlider.maximumValue
    property real _fireOutput
    property real valueScale: 1e-6

    width: parent.width

    Text {
        text: "Stimulation output: " + _fireOutput.toFixed(1) + " uS"
    }
    CheckBox {
        id: inhibitoryCheckbox
        text: "Inhibitory"
    }
    Slider {
        id: fireOutputSlider
        width: parent.width
        minimumValue: 0.0
        maximumValue: 400.0
        stepSize: 1.0
    }
    Binding {
        target: root.target
        property: root.property
        value: (inhibitoryCheckbox.checked ? -1.0 : 1.0) * fireOutputSlider.value * valueScale
    }
    Binding {
        target: fireOutputSlider
        property: "value"
        value: Math.abs(root.target[root.property]) / valueScale
    }
    Binding {
        target: inhibitoryCheckbox
        property: "checked"
        value: root.target[root.property] < 0.0
    }
    Binding {
        target: root
        property: "_fireOutput"
        value: root.target[root.property] / valueScale
    }
}

