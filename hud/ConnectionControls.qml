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
        conductanceSlider.value = conductanceSlider.inverseScaledConductance(connection.conductance)
    }

    revealed: connectionControlsRoot.connection ? true : false
    ColumnLayout {

        anchors.fill: parent
        spacing: 10
        anchors.margins: 10

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
