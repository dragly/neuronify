import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Controls.Styles 1.1
import ".."

PropertiesPanel {
    id: neuronControlsRoot

    signal disconnectClicked
    signal deleteClicked

    property Neuron neuron: null
    revealed: neuronControlsRoot.neuron ? true : false
    onNeuronChanged: {
        if(!neuronControlsRoot.neuron) {
            return
        }
        synapticOutputSlider.value = neuron.outputStimulation
        adaptationIncreaseOnFireSlider.value = neuron.adaptationIncreaseOnFire
        //        targetVoltageSlider.value = neuron.targetVoltage
        //        targetVoltageCheckbox.checked = neuron.forceTargetVoltage
        clampCurrentSlider.value = neuron.clampCurrent
        clampCurrentCheckbox.checked = neuron.clampCurrentEnabled
    }
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 10

        Text {
            text: "Polarization jump: " + polarizationSlider.value.toFixed(1) + " mV"
        }

        Slider {
            id: polarizationSlider
            minimumValue: -100
            maximumValue: 100
            Layout.fillWidth: true
        }

        Button {
            id: polarizeButton
            Layout.fillWidth: true

            text: "Depolarize!"
            onClicked: {
                neuronControlsRoot.neuron.voltage += polarizationSlider.value
            }
        }

        Text {
            text: "Synaptic output: " + synapticOutputSlider.value.toFixed(1) + " mS "
                  + (synapticOutputSlider.value > 0.0 ? "(excitatory)" : "(inhibitory)")
        }

        Slider {
            id: synapticOutputSlider
            minimumValue: -5.0
            maximumValue: 10.0
            stepSize: 0.1
            tickmarksEnabled: true
            Layout.fillWidth: true
            onValueChanged: {
                if(!neuronControlsRoot.neuron) {
                    return
                }
                neuronControlsRoot.neuron.outputStimulation = value
            }
        }

        //        CheckBox {
        //            id: targetVoltageCheckbox
        //            text: "Clamp voltage:" + targetVoltageSlider.value.toFixed(1) + " mV"
        //            onCheckedChanged: {
        //                if(!neuronControlsRoot.neuron) {
        //                    return
        //                }
        //                neuronControlsRoot.neuron.forceTargetVoltage = checked
        //                neuronControlsRoot.neuron.targetVoltage = targetVoltageSlider.value
        //            }
        //        }

        //        Slider {
        //            id: targetVoltageSlider
        //            minimumValue: -100.0
        //            maximumValue: 100.0
        //            stepSize: 1.0
        //            tickmarksEnabled: true
        //            Layout.fillWidth: true
        //            onValueChanged: {
        //                if(!neuronControlsRoot.neuron) {
        //                    return
        //                }
        //                neuronControlsRoot.neuron.targetVoltage = value
        //            }
        //        }

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
        Text {
            text: "Adaptation increase on fire: " + adaptationIncreaseOnFireSlider.value.toFixed(1) + " mS"
        }
        Slider {
            id: adaptationIncreaseOnFireSlider
            minimumValue: -10.0
            maximumValue: 100.0
            stepSize: 1.0
            tickmarksEnabled: true
            Layout.fillWidth: true
            onValueChanged: {
                if(!neuronControlsRoot.neuron) {
                    return
                }

                neuronControlsRoot.neuron.adaptationIncreaseOnFire = value
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
                deleteClicked()
            }
        }
        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}
