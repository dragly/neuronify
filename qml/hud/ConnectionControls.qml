import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import ".."

Item {
    id: connectionControlsRoot
    property Item connection: null

    signal deleteClicked

    anchors.fill: parent

    onConnectionChanged: {
        if(!connectionControlsRoot.connection) {
            return
        }
    }

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
