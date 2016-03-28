import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import ".."
import "../controls"

Column {
    id: sensorControlsRoot
    property Item sensor: null

    signal deleteClicked

    spacing: 10
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
}

