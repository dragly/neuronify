import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Controls.Styles 1.1
import ".."

Item {
    id: neuronControlsRoot

    signal disconnectClicked
    signal deleteClicked

    property Item neuron: null

    anchors.fill: parent

    onNeuronChanged: {
        if(!neuronControlsRoot.neuron) {
            return
        }
        synapticOutputSlider.value = neuron.stimulation
        clampCurrentSlider.value = neuron.clampCurrent
        clampCurrentCheckbox.checked = neuron.clampCurrentEnabled
        inhibitoryCheckbox.checked = neuron.stimulation < 0
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
                neuronControlsRoot.neuron.voltage += 100
            }
        }


        CheckBox {
            id: inhibitoryCheckbox
            text: "Inhibitory"
            checked: false

            onCheckedChanged: {
                if(!neuronControlsRoot.neuron) {
                                    return

                }

                synapticOutputSlider.value = Math.abs(neuronControlsRoot.neuron.stimulation)

                if (inhibitoryCheckbox.checked) {
                    neuronControlsRoot.neuron.stimulation = -synapticOutputSlider.value
                } else{
                    neuronControlsRoot.neuron.stimulation = synapticOutputSlider.value
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
                if(!neuronControlsRoot.neuron) {
                    return
                }
                if (inhibitoryCheckbox.checked) {
                    neuronControlsRoot.neuron.stimulation = -value
                } else{
                    neuronControlsRoot.neuron.stimulation = value
                }
            }
        }

        CheckBox {
            id: clampCurrentCheckbox
            text: "Current clamp: " + clampCurrentSlider.value.toFixed(1) + " mA"
            onCheckedChanged: {
                if(!neuronControlsRoot.neuron) {
                    return
                }
                neuronControlsRoot.neuron.clampCurrentEnabled = checked
                neuronControlsRoot.neuron.clampCurrent = clampCurrentSlider.value
            }
        }

        Slider {
            id: clampCurrentSlider
            minimumValue: -100.0
            maximumValue: 100.0
            stepSize: 1.0
            tickmarksEnabled: true
            Layout.fillWidth: true
            onValueChanged: {
                if(!neuronControlsRoot.neuron) {
                    return
                }

                neuronControlsRoot.neuron.clampCurrent = value
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
