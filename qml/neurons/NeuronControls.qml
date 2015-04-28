import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Controls.Styles 1.1

import Neuronify 1.0

import ".."
import "../controls"

Item {
    id: neuronControlsRoot

    signal deleteClicked

    property NeuronEngine engine: null

    anchors.fill: parent

    ColumnLayout {
        anchors.fill: parent
        spacing: 10

        Text {
            text: engine.voltage.toFixed(0) + " mV"
            anchors.right: parent.right
        }

        Text {
            text: "Label:"
        }
        TextField {
            text: ""
            onAccepted: {
                label = text
            }
        }

        FireOutputControl {
            target: engine
        }

        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}
