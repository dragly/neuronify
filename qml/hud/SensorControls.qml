import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import ".."
import "../controls"

Item {
    id: sensorControlsRoot
    property Item sensor: null

    signal deleteClicked

    anchors.fill: parent

    ColumnLayout {
        anchors.fill: parent
        spacing: 10

        Text {
            text: "Cells: " + sensor.cells
        }
        BoundSlider {
            Layout.fillWidth: true
            minimumValue: 1
            maximumValue: 10
            stepSize: 1
            target: sensor
            property: "cells"
        }

        Text {
            text: "Current output: " + sensor.sensingCurrentOutput.toFixed(1) + " mA"
        }
        BoundSlider {
            Layout.fillWidth: true
            minimumValue: 0.0
            maximumValue: 1000
            target: sensor
            property: "sensingCurrentOutput"
        }

        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}

