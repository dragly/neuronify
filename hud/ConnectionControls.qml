import QtQuick 2.0
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.1
import ".."

PropertiesPanel {
    id: connectionControlsRoot
    property Connection connection: null

    signal deleteClicked

    onConnectionChanged: {
        if(!connectionControlsRoot.connection) {
            return
        }
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
                connectionControlsRoot.deleteClicked()
            }
        }

        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}
