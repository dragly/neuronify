import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Controls.Styles 1.1

import Neuronify 1.0

import "qrc:/qml"
import "qrc:/qml/controls"
import "qrc:/qml/style"

Column {
    property var neuron: null
    property NeuronEngine engine: null

    spacing: Style.control.spacing
    width: parent ? parent.width : 100

    Text {
        text: (neuron.voltage * 1e3).toFixed(0) + " mV"
        anchors.right: parent.right
    }

    Text {
        text: "Label:"
    }

    TextField {
        id: labelField
        text: neuron.label
        anchors {
            left: parent.left
            right: parent.right
        }
    }

    Binding {
        target: neuron
        property: "label"
        value: labelField.text
    }

    Text {
        text: "Neuron dynamics:"
    }

    BoundSlider {       
        target: engine
        property: "restingPotential"
        text: "Resting potential"
        unit: "mV"
        minimumValue: -100e-3
        maximumValue: 50e-3
        unitScale: 1e-3
        stepSize: 1e-4
        precision: 1
    }

    BoundSlider {
        target: engine
        property: "threshold"
        text: "Firing threshold"
        unit: "mV"
        minimumValue: -50e-3
        maximumValue: 50e-3
        unitScale: 1e-3
        stepSize: 1e-4
        precision: 1
    }

    BoundSlider {
        target: engine
        property: "initialPotential"
        text: "Initial potential"
        unit: "mV"
        minimumValue: -100e-3
        maximumValue: 50e-3
        unitScale: 1e-3
        stepSize: 1e-4
        precision: 1
    }

    BoundSlider {
        target: engine
        property: "capacitance"
        text: "Capacitance"
        unit: "nF"
        minimumValue: 1.0e-9
        maximumValue: 10000e-9
        unitScale: 1e-9
        stepSize: 1e-8
        precision: 1
    }

    BoundSlider {
        target: engine
        property: "refractoryPeriod"
        text: "Refractory period"
        unit: "ms"
        minimumValue: 0.0e-3
        maximumValue: 100e-3
        unitScale: 1e-3
        stepSize: 1e-3
        precision: 1
    }

    Text {
        text: "Synaptic input:"
    }

    BoundSlider {
        target: engine
        property: "synapticPotential"
        text: "Potential"
        unit: "mV"
        minimumValue: -100e-3
        maximumValue: 50e-3
        unitScale: 1e-3
        stepSize: 1e-4
        precision: 1
    }


    BoundSlider {
        target: engine
        property: "synapticTimeConstant"
        text: "Time Constant"
        unit: "ms"
        minimumValue: 0.0
        maximumValue: 50e-3
        unitScale: 1e-3
        stepSize: 1e-4
        precision: 1
    }

    Text {
        text: "Synaptic output:"
    }

    BoundSlider {
        target: engine
        property: "fireOutput"
        minimumValue: -100.0e-6
        maximumValue: 100.0e-6
        unitScale: 1e-6
        text: "Stimulation"
        unit: "uS"
        stepSize: 1.0e-6
    }
}
