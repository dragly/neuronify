import QtQuick 2.0
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.1
import ".."

PropertiesPanel {
    id: voltmeterControlsRoot
    property Voltmeter voltmeter: null

    onVoltmeterChanged: {
        if(!voltmeterControlsRoot.voltmeter) {
            return
        }
        var voltmeter = voltmeterControlsRoot.voltmeter
        switch(voltmeter.mode) {
        case "voltage":
            voltageRadioButton.checked = true
            break
        case "sodiumCurrent":
            sodiumCurrentRadioButton.checked = true
            break
        case "potassiumCurrent":
            potassiumCurrentRadioButton.checked = true
            break
        case "leakCurrent":
            leakCurrentRadioButton.checked = true
            break
        }
    }

    revealed: voltmeterControlsRoot.voltmeter ? true : false
    ColumnLayout {

        anchors.fill: parent
        spacing: 10
        anchors.margins: 10

        Text {
            text: "Mode:"
        }

        ExclusiveGroup {
            id: modeGroup
        }

        RadioButton {
            id: voltageRadioButton
            text: "Voltage"
            exclusiveGroup: modeGroup
            onCheckedChanged: {
                if(checked) {
                    voltmeterControlsRoot.voltmeter.mode = "voltage"
                }
            }
        }

        RadioButton {
            id: sodiumCurrentRadioButton
            text: "Sodium current"
            exclusiveGroup: modeGroup
            onCheckedChanged: {
                if(checked) {
                    voltmeterControlsRoot.voltmeter.mode = "sodiumCurrent"
                }
            }
        }

        RadioButton {
            id: potassiumCurrentRadioButton
            text: "Potassium current"
            exclusiveGroup: modeGroup
            onCheckedChanged: {
                if(checked) {
                    voltmeterControlsRoot.voltmeter.mode = "potassiumCurrent"
                }
            }
        }

        RadioButton {
            id: leakCurrentRadioButton
            text: "Leak current"
            exclusiveGroup: modeGroup
            onCheckedChanged: {
                if(checked) {
                    voltmeterControlsRoot.voltmeter.mode = "leakCurrent"
                }
            }
        }

        Button {
            text: "Delete"
            onClicked: {
                simulatorRoot.deleteVoltmeter(voltmeterControlsRoot.voltmeter)
            }
        }

        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}
