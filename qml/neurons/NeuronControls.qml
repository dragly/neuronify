import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Controls.Styles 1.1

import Neuronify 1.0

import ".."
import "../controls"

Column {
    signal deleteClicked

    property var neuron: null
    property NeuronEngine engine: null

    width: parent ? parent.width : 100
    spacing: 10

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

    FireOutputControl {
        target: engine
    }
    
    Text {
        text: "Reset the potential:"
    }

    Button {
        text: "Reset"
        onClicked: {
            engine.resetVoltage()
        }
    }

    Text {
        text: "Reset the potential of all neurons:"
    }

    Button {
        text: "Reset all"
        onClicked: {
            for (var i in graphEngine.nodes){
                if (graphEngine.nodes[i].isNeuron) {
                    graphEngine.nodes[i].engine.resetVoltage()
                }

            }
        }
    }
}
