import QtQuick 2.0
import QtQuick.Controls 1.2

Rectangle {
    id: simulatorRoot
    width: 400
    height: 300
    property var compartments: []

    Component.onCompleted: {
        var previousCompartment = undefined
        for(var i = 0; i < 10; i++) {
            var component = Qt.createComponent("Compartment.qml")
            var compartment = component.createObject(simulatorRoot, {x: i * 100})
            if(previousCompartment) {
                previousCompartment.connections.push(compartment)
                compartment.connections.push(previousCompartment)
            }
            compartments.push(compartment)
            previousCompartment = compartment
        }
    }

    Column {
        anchors {
            right: parent.right
            top: parent.top
        }
        Slider {
            id: polarizationSlider
            minimumValue: -100
            maximumValue: 100
        }

        Text {
            text: "Polarization jump: " + polarizationSlider.value.toFixed(1) + " mV"
        }

        Button {
            id: polarizeButton

            text: "Polarize!"
            onClicked: {
                compartments[0].voltage += polarizationSlider.value
            }
        }

        Button {
            id: resetButton

            text: "Reset!"
            onClicked: {
                for(var i in compartments) {
                    var compartment = compartments[i]
                    compartment.reset()
                }
            }
        }

        CheckBox {
            id: targetVoltageCheckbox
            text: "Lock to target voltage"
        }

        Text {
            text: "Target voltage: " + targetVoltageSlider.value.toFixed(1) + " mV"
        }

        Slider {
            id: targetVoltageSlider
            minimumValue: -120
            maximumValue: 80.0
        }
    }

    Plot {
        id: plot
        strokeStyle: "blue"
    }

    Plot {
        id: plot2
        strokeStyle: "green"
    }

    Timer {
        interval: 1
        running: true
        repeat: true
        onTriggered: {
            for(var i in compartments) {
                var compartment = compartments[i]
                compartment.stepForward()
            }
            for(var i in compartments) {
                var compartment = compartments[i]
                compartment.finalizeStep()
            }
            plot.addPoint(compartments[0].voltage)
            plot2.addPoint(compartments[compartments.length - 1].voltage)
        }
    }
}
