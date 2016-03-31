import QtQuick 2.0
import QtQuick.Controls 1.4

import Neuronify 1.0

import "qrc:/qml"
import "qrc:/qml/controls"
import "qrc:/qml/neurons"
import "qrc:/qml/style"

Column {
    property Neuron neuron
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
    Binding {
        target: labelField
        property: "text"
        value: neuron.label
    }
}
