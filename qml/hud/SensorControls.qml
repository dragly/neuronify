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
        BoundSlider {
            Layout.fillWidth: true
            minimumValue: 1
            maximumValue: 10
            stepSize: 1
            text: "Sensor cells"
            target: sensor
            property: "cells"
        }
        BoundSlider {
            Layout.fillWidth: true
            minimumValue: 0.0e-6
            maximumValue: 50e-6
            unitScale: 1e-6
            stepSize: 1e-7
            text: "Current output"
            unit: "µA"
            target: sensor
            property: "sensingCurrentOutput"
        }

        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}

