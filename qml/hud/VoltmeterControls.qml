import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1

import ".."
import "../controls"

/*!
    \qmltype VoltMeterControls
    \brief Contains the user controls for the voltmeter

    The voltmeter control panel contains
\list
 \li \l A mode selection (Voltage, conductance, etc)
 \li \l A "connect to all"-button which connects the voltmeter to all existing neurons
 \li \l Sliders for setting the minimum and maximum voltage
\endlist
*/

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
                    if (itemB.isNeuron){
                        if (!voltmeter.simulator.connectionExists(itemB, itemA)){
                            voltmeter.simulator.connectEntities(itemB, itemA)
                        }
                    }
                }
            }
        }

        BoundSlider {
            target: voltmeter
            property: "minimumValue"
            text: "Minimum voltage"
            unit: "mV"
            minimumValue: -250
            maximumValue: 250
        }

        BoundSlider {
            target: voltmeter
            property: "maximumValue"
            text: "Maximum voltage"
            unit: "mV"
            minimumValue: -250
            maximumValue: 250
        }

        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}
