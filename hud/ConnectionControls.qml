import QtQuick 2.0
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.1
import ".."

PropertiesPanel {
    id: connectionControlsRoot
    property Connection connection: null

    onConnectionChanged: {
        if(!connectionControlsRoot.connection) {
            return
        }

        axialConductanceSlider.value = connection.axialConductance
    }

    revealed: connectionControlsRoot.connection ? true : false
    ColumnLayout {

        anchors.fill: parent
        spacing: 10

        Text {
            text: "Axial conductance: " + axialConductanceSlider.value.toFixed(2)
        }

        Slider {
            id: axialConductanceSlider
            minimumValue: 0.01
            maximumValue: 100
            onValueChanged: {
                if(!connectionControlsRoot.connection) {
                    return
                }
                connectionControlsRoot.connection.axialConductance = axialConductanceSlider.value
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
