import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Controls.Styles 1.1
import ".."

PropertiesPanel {
    id: compartmentControlsRoot

    signal disconnectClicked
    signal deleteClicked

    property Compartment compartment: null
    revealed: compartmentControlsRoot.compartment ? true : false
    onCompartmentChanged: {
        if(!compartmentControlsRoot.compartment) {
            return
        }
        targetVoltageSlider.value = compartment.targetVoltage
        targetVoltageCheckbox.checked = compartment.forceTargetVoltage
        passiveCheckbox.checked = compartmentControlsRoot.compartment.passive
        lengthSlider.value = compartment.length
        diameterSlider.value = compartment.diameter
        clampCurrentSlider.value = compartment.clampCurrent
        clampCurrentCheckbox.checked = compartment.clampCurrentEnabled
    }
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 10

        Text {
            text: "Length: " + lengthSlider.value.toFixed(1)
        }

        Slider {
            id: lengthSlider
            Layout.fillWidth: true
            minimumValue: 0.6
            maximumValue: 1.8
            onValueChanged: {
                if(!compartmentControlsRoot.compartment) {
                    return
                }
                compartmentControlsRoot.compartment.length = value
            }
        }

        Text {
            text: "Diameter: " + diameterSlider.value.toFixed(1)
        }

        Slider {
            id: diameterSlider
            Layout.fillWidth: true
            minimumValue: 0.5
            maximumValue: 1.4
            onValueChanged: {
                if(!compartmentControlsRoot.compartment) {
                    return
                }
                compartmentControlsRoot.compartment.diameter = value
            }
        }

        CheckBox {
            id: passiveCheckbox
            text: "Passive"
            onCheckedChanged: {
                compartmentControlsRoot.compartment.passive = checked
            }
        }

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
                compartmentControlsRoot.compartment.voltage += polarizationSlider.value
            }
        }

        CheckBox {
            id: targetVoltageCheckbox
            text: "Clamp voltage:" + targetVoltageSlider.value.toFixed(1) + " mV"
            onCheckedChanged: {
                if(!compartmentControlsRoot.compartment) {
                    return
                }
                compartmentControlsRoot.compartment.forceTargetVoltage = checked
                compartmentControlsRoot.compartment.targetVoltage = targetVoltageSlider.value
            }
        }

        Slider {
            id: targetVoltageSlider
            minimumValue: -100.0
            maximumValue: 100.0
            stepSize: 1.0
            tickmarksEnabled: true
            Layout.fillWidth: true
            onValueChanged: {
                if(!compartmentControlsRoot.compartment) {
                    return
                }
                compartmentControlsRoot.compartment.targetVoltage = value
            }
        }

        CheckBox {
            id: clampCurrentCheckbox
            text: "Current clamp: " + clampCurrentSlider.value.toFixed(1) + " mA"
            onCheckedChanged: {
                if(!compartmentControlsRoot.compartment) {
                    return
                }
                compartmentControlsRoot.compartment.clampCurrentEnabled = checked
                compartmentControlsRoot.compartment.clampCurrent = clampCurrentSlider.value
            }
        }

        Text {
            text: "Current clamp: " + clampCurrentSlider.value.toFixed(1) + " mA"
        }

        Slider {
            id: clampCurrentSlider
            minimumValue: -100.0
            maximumValue: 100.0
            stepSize: 1.0
            tickmarksEnabled: true
            Layout.fillWidth: true
            onValueChanged: {
                if(!compartmentControlsRoot.compartment) {
                    return
                }

                compartmentControlsRoot.compartment.clampCurrent = value
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
