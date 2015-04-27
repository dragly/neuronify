import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import ".."


Item {
    id: voltmeterControlsRoot
    property Item voltmeter: null


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


    anchors.fill: parent

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
            text: "Connect to all neurons"
            onClicked: {
                var itemA = voltmeter
                for (var i in voltmeter.simulator.graphEngine.nodes){
                    var itemB = graphEngine.nodes[i]
                    if (itemB.objectName.indexOf("neuron") + itemB.objectName.indexOf("Neuron") != -2){
                        if (!voltmeter.simulator.connectionExists(itemB, itemA)){
                            voltmeter.simulator.connectEntities(itemB, itemA)
                        }
                    }
                }
            }
        }

        Button {
            text: "Delete"
            onClicked: {
                voltmeterControlsRoot.voltmeter.destroy()
            }
        }

        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}
