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
        text: engine.voltage.toFixed(0) + " mV"
        anchors.right: parent.right
    }

    Text {
        text: "Label:"
    }

    TextField {
        id: labelField
        text: neuron.label
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
        precision: 0
        minimumValue: -100
        maximumValue: 50
        //onValueChanged: object.value = value

    }

    BoundSlider {
        target: engine
        property: "threshold"
        text: "Firing threshold"
        minimumValue: -50
        maximumValue: 50
        precision: 0
        unit: "mV"
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
            for (var i in selectedEntities){
                if (selectedEntities[i].objectName.slice(-6) == "Neuron") {
                    selectedEntities[i].engine.resetVoltage()
                }
            }
        }
    }
}
