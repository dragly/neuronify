import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import ".."

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
        Slider {
            id: cellsSlider
            Layout.fillWidth: true
            minimumValue: 1
            maximumValue: 10
            stepSize: 1
            value: sensor.cells
        }
        Binding {
            target: sensor
            property: "cells"
            value: cellsSlider.value
        }

        Text {
            text: "Current output: " + sensor.sensingCurrentOutput.toFixed(1) + " mA"
        }
        Slider {
            id: sensingCurrentOutputSlider
            Layout.fillWidth: true
            minimumValue: 0.0
            maximumValue: 1000
            value: sensor.sensingCurrentOutput
        }
        Binding {
            target: sensor
            property: "sensingCurrentOutput"
            value: sensingCurrentOutputSlider.value
        }

        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}

