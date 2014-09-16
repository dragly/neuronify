import QtQuick 2.0
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.1
import ".."

PropertiesPanel {
    id: connectionControlsRoot
    property Connection connection: null

    color: "#deebf7"
    border.color: "#9ecae1"
    border.width: 1.0

    onConnectionChanged: {
        if(!connectionControlsRoot.connection) {
            return
        }
        conductanceSlider.value = conductanceSlider.inverseScaledConductance(connection.conductance)
    }

    revealed: connectionControlsRoot.connection ? true : false
    ColumnLayout {

        anchors.fill: parent
        spacing: 10

        Text {
            text: "Connection conductance: " + conductanceSlider.scaledConductance(conductanceSlider.value).toFixed(2)
        }

        Slider {
            id: conductanceSlider

            function scaledConductance(value) {
                return Math.exp(value)
            }

            function inverseScaledConductance(value) {
                return Math.log(value)
            }

            minimumValue: inverseScaledConductance(0.01)
            maximumValue: inverseScaledConductance(50)
            onValueChanged: {
                if(!connectionControlsRoot.connection) {
                    return
                }
                connectionControlsRoot.connection.conductance = scaledConductance(conductanceSlider.value)
            }
        }

        Button {
            text: "Delete"
            onClicked: {
                if(!connectionControlsRoot.connection) {
                    return
                }
                simulatorRoot.deleteConnection(connectionControlsRoot.connection)
            }
        }

        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}
