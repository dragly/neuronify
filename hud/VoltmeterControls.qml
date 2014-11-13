import QtQuick 2.0
import QtQuick.Controls 1.1
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
