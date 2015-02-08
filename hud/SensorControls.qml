import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import ".."

PropertiesPanel {
    id: sensorControlsRoot
    property TouchSensor sensor: null

    signal deleteClicked

    onSensorChanged: {
        if(!sensorControlsRoot.sensor) {
            return
        }
        cellsSlider.value = sensorControlsRoot.sensor.cells
    }

    revealed: sensorControlsRoot.sensor ? true : false
    ColumnLayout {

        anchors.fill: parent
        spacing: 10
        anchors.margins: 10

        Text {
            text: "Cells: " + cellsSlider.value.toFixed(0)
        }

        Slider {
            id: cellsSlider
            Layout.fillWidth: true
            minimumValue: 1
            maximumValue: 10
            stepSize: 1
            onValueChanged: {
                if(!sensorControlsRoot.sensor) {
                    return
                }

                sensorControlsRoot.sensor.cells = cellsSlider.value
            }
        }

        Button {
            text: "Delete"
            onClicked: {
                if(!sensorControlsRoot.sensor) {
                    return
                }
                sensorControlsRoot.deleteClicked()
            }
        }

        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}

