import QtQuick 2.0
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.1
import ".."

PropertiesPanel {
    id: compartmentControlsRoot
    property Compartment compartment: null
    revealed: compartmentControlsRoot.compartment ? true : false
    onCompartmentChanged: {
        if(!compartmentControlsRoot.compartment) {
            return
        }
        targetVoltageCheckbox.checked = compartment.forceTargetVoltage
    }
    ColumnLayout {
        anchors.fill: parent
        spacing: 10

        Slider {
            id: polarizationSlider
            minimumValue: -100
            maximumValue: 100
            Layout.fillWidth: true
        }

        Text {
            text: "Polarization jump: " + polarizationSlider.value.toFixed(1) + " mV"
        }

        Button {
            id: polarizeButton
            Layout.fillWidth: true

            text: "Polarize!"
            onClicked: {
                compartmentControlsRoot.compartment.voltage += polarizationSlider.value
            }
        }

        Button {
            id: resetButton
            Layout.fillWidth: true

            text: "Reset!"
            onClicked: {
                for(var i in compartments) {
                    var compartment = compartments[i]
                    compartment.reset()
                }
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
                simulatorRoot.disconnectCompartment(compartmentControlsRoot.compartment)
            }
        }

        Button {
            text: "Delete"
            Layout.fillWidth: true
            onClicked: {
                simulatorRoot.deleteCompartment(compartmentControlsRoot.compartment)
            }
        }
        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}
