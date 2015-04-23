import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Controls.Styles 1.1

import Neuronify 1.0

import ".."

Item {
    id: neuronControlsRoot

    signal disconnectClicked
    signal deleteClicked

    property NeuronEngine engine: null

    anchors.fill: parent

    onEngineChanged: {
        if(!neuronControlsRoot.engine) {
            return
        }
        synapticOutputSlider.value = engine.stimulation
        inhibitoryCheckbox.checked = engine.stimulation < 0
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 10

        Button {
            id: polarizeButton
            Layout.fillWidth: true

            text: "Fire!"
            onClicked: {
                neuronControlsRoot.engine.voltage += 100
            }
        }


        CheckBox {
            id: inhibitoryCheckbox
            text: "Inhibitory"
            checked: false

            onCheckedChanged: {
                if(!neuronControlsRoot.engine) {
                    return
                }

                synapticOutputSlider.value = Math.abs(neuronControlsRoot.engine.stimulation)

                if (inhibitoryCheckbox.checked) {
                    neuronControlsRoot.engine.stimulation = -synapticOutputSlider.value
                } else{
                    neuronControlsRoot.engine.stimulation = synapticOutputSlider.value
                }
            }
        }


        Text {
            text: "Synaptic output: " + (inhibitoryCheckbox.checked ? " -" : "  ") + synapticOutputSlider.value.toFixed(1) + " mS "
        }


        Slider {
            id: synapticOutputSlider
            minimumValue: 0.
            maximumValue: 10.
            stepSize: 0.1
            tickmarksEnabled: true
            Layout.fillWidth: true

            onValueChanged: {
                if(!neuronControlsRoot.engine) {
                    return
                }
                if (inhibitoryCheckbox.checked) {
                    neuronControlsRoot.engine.stimulation = -value
                } else{
                    neuronControlsRoot.engine.stimulation = value
                }
            }
        }

        Button {
            id: disconnectButton
            text: "Disconnect"
            Layout.fillWidth: true
            onClicked: {
                disconnectClicked()
            }
        }

        Button {
            text: "Delete"
            Layout.fillWidth: true
            onClicked: {
                console.log("Calling deleteClicked signal!")
                deleteClicked()
            }
        }
        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}
