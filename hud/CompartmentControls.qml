import QtQuick 2.0
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.1
import QtQuick.Controls.Styles 1.2
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

            text: "Polarize!"
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
                compartmentControlsRoot.compartment.targetVoltage = value
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
