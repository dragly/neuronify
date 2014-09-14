import QtQuick 2.0
import QtQuick.Controls 1.2

Rectangle {
    id: simulatorRoot

    property real lastOrganizeTime: Date.now()
    property var compartments: []
    property var connections: []

    Component.onCompleted: {
        var previousCompartment = undefined
        var previousCompartment2 = undefined
        for(var i = 0; i < 10; i++) {
            var component = Qt.createComponent("Compartment.qml")
            var compartment = component.createObject(compartmentLayer, {x: i * 5, y: i * 10})
            compartment.dragStarted.connect(resetOrganize)
            if(previousCompartment) {
                var connectionComponent = Qt.createComponent("Connection.qml")
                var connection = connectionComponent.createObject(connectionLayer, {
                                                                      sourceCompartment: previousCompartment,
                                                                      targetCompartment: compartment
                                                                  })

                previousCompartment.connections.push(connection)
                compartment.connections.push(connection)
                connections.push(connection)
            }
            if(previousCompartment2) {
                var connectionComponent = Qt.createComponent("Connection.qml")
                var connection = connectionComponent.createObject(connectionLayer, {
                                                                      sourceCompartment: previousCompartment2,
                                                                      targetCompartment: compartment
                                                                  })

                previousCompartment2.connections.push(connection)
                compartment.connections.push(connection)
                connections.push(connection)
            }
            compartments.push(compartment)
            previousCompartment2 = previousCompartment
            previousCompartment = compartment
        }
    }

    function resetOrganize() {
        lastOrganizeTime = Date.now()
        layoutTimer.start()
    }

    function organize() {
        var currentOrganizeTime = Date.now()
        var dt = (currentOrganizeTime - lastOrganizeTime) / 1000.0
        var springLength = simulatorRoot.width * 0.1
        var anyDragging = false

        for(var i in compartments) {
            var compartment = compartments[i]
            compartment.velocity = Qt.vector2d(0,0)
            if(compartment.dragging) {
                anyDragging = true
            }
        }

        for(var i in connections) {
            var connection = connections[i]
            var source = connection.sourceCompartment
            var target = connection.targetCompartment
            var xDiff = source.x - target.x
            var yDiff = source.y - target.y
            var length = Math.sqrt(xDiff*xDiff + yDiff*yDiff)
            var lengthDiff = length - springLength
            var xDelta = lengthDiff * xDiff / length
            var yDelta = lengthDiff * yDiff / length
            var kFactor = lengthDiff > 0 ? 0.02 : 0.01
            var k = kFactor * simulatorRoot.width
            if(!source.dragging) {
                source.velocity.x -= 0.5 * k * xDelta
                source.velocity.y -= 0.5 * k * yDelta
            }
            if(!target.dragging) {
                target.velocity.x += 0.5 * k * xDelta
                target.velocity.y += 0.5 * k * yDelta
            }
        }

        for(var i = 0; i < compartments.length; i++) {
            var minDistance = springLength * 0.8
            var compartmentA = compartments[i]
            for(var j = i + 1; j < compartments.length; j++) {
                var compartmentB = compartments[j]
                var xDiff = compartmentA.x - compartmentB.x
                var yDiff = compartmentA.y - compartmentB.y
                var length = Math.sqrt(xDiff*xDiff + yDiff*yDiff)
                if(length > minDistance) {
                    continue
                }

                var lengthDiff = length - minDistance
                var xDelta = lengthDiff * xDiff / length
                var yDelta = lengthDiff * yDiff / length
                var k = simulatorRoot.width * 0.01
                if(!compartmentA.dragging) {
                    compartmentA.velocity.x -= 0.5 * k * xDelta
                    compartmentA.velocity.y -= 0.5 * k * yDelta
                }
                if(!compartmentB.dragging) {
                    compartmentB.velocity.x += 0.5 * k * xDelta
                    compartmentB.velocity.y += 0.5 * k * yDelta
                }
            }
        }

        var maxAppliedVelocity = 0.0
        var maxVelocity = simulatorRoot.width * 1.0
        for(var i in compartments) {
            var compartment = compartments[i]
            var velocity = Math.sqrt(compartment.velocity.x*compartment.velocity.x + compartment.velocity.y*compartment.velocity.y)
            if(velocity > maxVelocity) {
                compartment.velocity.x *= (maxVelocity / velocity)
            }

            maxAppliedVelocity = Math.max(maxAppliedVelocity, compartment.velocity.x*compartment.velocity.x + compartment.velocity.y*compartment.velocity.y)
            compartment.x += compartment.velocity.x * dt
            compartment.y += compartment.velocity.y * dt
        }

        if(maxAppliedVelocity < simulatorRoot.width * 0.01 && !anyDragging) {
            console.log("Stopping timer with max velocity: " + maxAppliedVelocity)
            layoutTimer.stop()
        }

        lastOrganizeTime = currentOrganizeTime
    }

    width: 400
    height: 300

    Item {
        id: connectionLayer
        anchors.fill: parent
    }

    Item {
        id: compartmentLayer
        anchors.fill: parent
    }

    Button {
        text: "Organize"
        onClicked: organize()
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

    Plot {
        id: plot3
        strokeStyle: "red"
    }

    Timer {
        id: layoutTimer
        interval: 24
        running: true
        repeat: true
        onTriggered: {
            organize()
        }
    }

    Timer {
        interval: 1
        running: true
        repeat: true
        onTriggered: {
            if(targetVoltageCheckbox.checked) {
                compartments[0].voltage = targetVoltageSlider.value
            }
            for(var i in compartments) {
                var compartment = compartments[i]
                compartment.stepForward()
            }
            if(targetVoltageCheckbox.checked) {
                compartments[0].voltage = targetVoltageSlider.value
            }
            for(var i in compartments) {
                var compartment = compartments[i]
                compartment.finalizeStep()
            }
            if(targetVoltageCheckbox.checked) {
                compartments[0].voltage = targetVoltageSlider.value
            }
            plot.addPoint(compartments[0].voltage)
            plot2.addPoint(compartments[1].voltage)
            plot3.addPoint(compartments[compartments.length - 1].voltage)
        }
    }
}
