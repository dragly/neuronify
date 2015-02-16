import QtQuick 2.0
import QtQuick.Layouts 1.1
import QtQuick.Controls 1.2

Rectangle {
    id: playbackControls

    property real speed: Math.pow(10, playbackSpeedSlider.value)

    anchors {
        bottom: parent.bottom
        left: parent.left
        right: parent.right
    }
    height: parent.height * 0.08

    color: "#deebf7"
    border.color: "#9ecae1"
    border.width: 1.0

    RowLayout {
        spacing: 10
        anchors.fill: parent
        anchors.margins: 10

        CheckBox {
            id: playingCheckbox
            text: "Simulate"
            checked: true
        }

        Text {
            text: "Speed: "
        }

        Slider {
            id: playbackSpeedSlider
            minimumValue: -1
            maximumValue: 1.2
            Layout.fillWidth: true
        }

        Text {
            text: playbackControls.speed.toFixed(1) + " x"
        }

        Button {
            id: resetButton

            text: "Reset!"
            onClicked: {
                for(var i in neurons) {
                    var neuron = neurons[i]
                    neuron.reset()
                }
            }
        }

        Text {
            text: "dt: " + currentTimeStep.toFixed(3)
        }

        Text {
            text: "Time: " + time.toFixed(3) + " ms"
        }

        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true
        }
    }
}

