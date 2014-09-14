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
            var compartment = createCompartment({x: 200 + i * 100, y: 200 + (Math.random()) * 10})
            if(previousCompartment) {
                createConnection(previousCompartment, compartment)
            }
//            if(previousCompartment2) {
//                createConnection(previousCompartment2, compartment)
//            }
            previousCompartment2 = previousCompartment
            previousCompartment = compartment
        }
    }

    function selectCompartment(compartment, mouse) {
        compartmentControls.compartment = compartment
        for(var i in compartments) {
            var otherCompartment = compartments[i]
            if(otherCompartment !== compartment) {
                otherCompartment.selected = false
            }
        }
        compartment.selected = true
    }

    function createCompartment(properties) {
        var component = Qt.createComponent("Compartment.qml")
        var compartment = component.createObject(compartmentLayer, properties)
        compartment.dragStarted.connect(resetOrganize)
        compartment.clicked.connect(selectCompartment)
        compartment.droppedConnectionCreator.connect(createConnectionToPoint)
        compartments.push(compartment)
        return compartment
    }

    function createConnection(sourceCompartment, targetCompartment) {
        var connectionComponent = Qt.createComponent("Connection.qml")
        var connection = connectionComponent.createObject(connectionLayer, {
                                                              sourceCompartment: sourceCompartment,
                                                              targetCompartment: targetCompartment
                                                          })

        sourceCompartment.connections.push(connection)
        targetCompartment.connections.push(connection)
        connections.push(connection)
        return connection
    }

    function createConnectionToPoint(sourceCompartment, connectionCreator) {
        var connectionCreated = false
        for(var i in compartments) {
            var targetCompartment = compartments[i]
            if(targetCompartment === sourceCompartment) {
                continue
            }
            var mouse2 = targetCompartment.mapFromItem(sourceCompartment, connectionCreator.x + connectionCreator.width / 2, connectionCreator.y + connectionCreator.height / 2)
            var tolerance = connectionCreator.width / 2 + targetCompartment.width * 0.1
            if(mouse2.x > -tolerance && mouse2.x < targetCompartment.width + tolerance
                    && mouse2.y > -tolerance && mouse2.y < targetCompartment.height + tolerance) {
                var connectionAlreadyExists = false
                for(var j in connections) {
                    var existingConnection = connections[j]
                    if((existingConnection.sourceCompartment === sourceCompartment && existingConnection.targetCompartment === targetCompartment)
                            || (existingConnection.targetCompartment === sourceCompartment && existingConnection.sourceCompartment === targetCompartment)) {
                        connectionAlreadyExists = true
                        break
                    }
                }
                if(connectionAlreadyExists) {
                    console.log("Connection already exists!")
                    break
                }

                createConnection(sourceCompartment, targetCompartment)
                connectionCreated = true
            }
        }
        if(connectionCreated) {
            resetOrganize()
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
        var minVelocity = simulatorRoot.width * 0.5
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

        if(maxAppliedVelocity < minVelocity && !anyDragging) {
            console.log("Stopping timer with max velocity: " + maxAppliedVelocity)
            layoutTimer.stop()
        }

        lastOrganizeTime = currentOrganizeTime
    }

    width: 400
    height: 300

    MouseArea {
        anchors.fill: parent
        onClicked: {
            compartmentControls.compartment = null
            for(var i in compartments) {
                var compartment = compartments[i]
                compartment.selected = false
            }
        }
    }

    Rectangle {
        id: compartmentCreator
        width: 60
        height: 40
        color: "red"
        MouseArea {
            anchors.fill: parent
            drag.target: parent
            onReleased: {
                createCompartment({x: compartmentCreator.x, y: compartmentCreator.y})
                compartmentCreator.x = 0
                compartmentCreator.y = 0
            }
        }
    }

    Item {
        id: connectionLayer
        anchors.fill: parent
    }

    Item {
        id: compartmentLayer
        anchors.fill: parent
    }

    Column {
        id: compartmentControls
        property Compartment compartment: null
        enabled: compartment ? true : false
        anchors {
            right: parent.right
            top: parent.top
        }

        onCompartmentChanged: {
            targetVoltageCheckbox.checked = compartment ? compartment.forceTargetVoltage : false
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
                compartmentControls.compartment.voltage += polarizationSlider.value
            }
        }

        Button {
            id: resetButton

            text: "Reset!"
            onClicked: {
                compartment.reset()
            }
        }

        CheckBox {
            id: targetVoltageCheckbox
            text: "Lock to target voltage"
            onCheckedChanged: {
                compartmentControls.compartment.forceTargetVoltage = checked
                compartmentControls.compartment.targetVoltage = targetVoltageSlider.value
            }
        }

        Text {
            text: "Target voltage: " + targetVoltageSlider.value.toFixed(1) + " mV"
        }

        Slider {
            id: targetVoltageSlider
            minimumValue: -120
            maximumValue: 80.0
            onValueChanged: {
                compartmentControls.compartment.targetVoltage = value
            }
        }

        Button {
            text: "Disconnect"
            onClicked: {
                var compartmentToDelete = compartmentControls.compartment
                var connectionsNew = connections
                var connectionsToDelete = compartmentToDelete.connections
                compartmentToDelete.connections = []
                var connectionsToDestroy = []
                for(var i in connectionsToDelete) {
                    var connection = connectionsToDelete[i]
                    var connectionIndex = connections.indexOf(connection)
                    if(connectionIndex > -1) {
                        connectionsNew.splice(connectionIndex, 1)
                    }
                    connection.targetCompartment.removeConnection(connection)
                    connection.sourceCompartment.removeConnection(connection)
                    connection.destroy()
                }
                connections = connectionsNew
            }
        }

        Button {
            text: "Delete"
            onClicked: {
                var compartmentToDelete = compartment
                compartment = null
                compartmentControls.compartment = null
                for(var i in connections) {
                    var connection = connections[i]

                }
            }
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
            for(var i in compartments) {
                var compartment = compartments[i]
                compartment.stepForward()
            }
            for(var i in compartments) {
                var compartment = compartments[i]
                compartment.finalizeStep()
            }
            plot.addPoint(compartments[0].voltage)
            plot2.addPoint(compartments[1].voltage)
            plot3.addPoint(compartments[compartments.length - 1].voltage)
        }
    }
}
