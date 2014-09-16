import QtQuick 2.0
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.1
import "hud"

Rectangle {
    id: simulatorRoot

    property real lastOrganizeTime: Date.now()
    property real lastStepTime: Date.now()
    property var compartments: []
    property var voltmeters: []
    property var voltmeterConnections: []
    property var compartmentConnections: []

    width: 400
    height: 300
    color: "#f7fbff"

    Component.onCompleted: {
        var previousCompartment = undefined
        var previousCompartment2 = undefined
        for(var i = 0; i < 5; i++) {
            var compartment = createCompartment({x: 300 + i * 100, y: 200 + (Math.random()) * 10})
            if(previousCompartment) {
                connectCompartments(previousCompartment, compartment)
            }
            if(i === 0) {
                compartment.targetVoltage = -12.0
                compartment.forceTargetVoltage = true
            }
            if(i === 1 || i === 4) {
                var voltmeter = createVoltmeter({x: 300 + i * 100, y: 400})
                connectVoltmeter(compartment, voltmeter)
            }
            previousCompartment = compartment
        }
    }

    function deleteCompartment(compartment) {
        disconnectCompartment(compartment)
        var compartmentsNew = simulatorRoot.compartments
        if(compartmentControls.compartment === compartment) {
            compartmentControls.compartment = null
        }
        var compartmentIndex = compartmentsNew.indexOf(compartment)
        if(compartmentIndex > -1) {
            compartmentsNew.splice(compartmentIndex, 1)
        }
        compartments = compartmentsNew
        compartment.destroy()
    }

    function deleteVoltmeter(voltmeter) {
        if(voltmeterControls.voltmeter === voltmeter) {
            voltmeterControls.voltmeter = null
        }
        var connectionsToRemove = []
        var connectionPlots = voltmeter.connectionPlots
        for(var i in connectionPlots) {
            var connectionPlot = connectionPlots[i]
            connectionsToRemove.push(connectionPlot.connection)
        }
        for(var i in connectionsToRemove) {
            deleteConnection(connectionsToRemove[i])
        }
        var voltmeterIndex = simulatorRoot.voltmeters.indexOf(voltmeter)
        if(voltmeterIndex > -1) {
            simulatorRoot.voltmeters.splice(voltmeterIndex, 1)
        }
        voltmeter.destroy()
    }

    function disconnectVoltmeter(voltmeter) {
        var connectionsToDelete = voltmeter.connectionPlots
        for(var i in connectionsToDelete) {
            var connection = connectionsToDelete[i]
            deleteConnection(connection)
        }
        var voltmeterConnectionsNew = voltmeterConnections
        for(var i in voltmeterConnections) {
            var voltmeterConnection = voltmeterConnections[i]
            if(voltmeterConnection.targetCompartment === voltmeter) {
                voltmeterConnectionsNew.splice(voltmeterConnections.indexOf(voltmeterConnection), 1)
                voltmeterConnection.destroy()
            }
        }
        voltmeterConnections = voltmeterConnectionsNew
    }

    function deleteConnection(connection) {
        var connectionIndex = compartmentConnections.indexOf(connection)
        if(connectionIndex > -1) {
            compartmentConnections.splice(connectionIndex, 1)
        }
        var voltmeterConnectionIndex = voltmeterConnections.indexOf(connection)
        if(voltmeterConnectionIndex > -1) {
            voltmeterConnections.splice(voltmeterConnectionIndex, 1)
        }
        connection.targetCompartment.removeConnection(connection)
        connection.sourceCompartment.removeConnection(connection)
        connection.destroy()
    }

    function disconnectCompartment(compartment) {
        var connectionsToDelete = compartment.connections
        compartment.connections = []
        var connectionsToDestroy = []
        for(var i in connectionsToDelete) {
            var connection = connectionsToDelete[i]
            deleteConnection(connection)
        }
        var voltmeterConnectionsNew = voltmeterConnections
        for(var i in voltmeterConnections) {
            var voltmeterConnection = voltmeterConnections[i]
            if(voltmeterConnection.sourceCompartment === compartment) {
                deleteConnection(voltmeterConnection)
            }
        }
        voltmeterConnections = voltmeterConnectionsNew
    }

    function isItemUnderConnector(item, source, connector) {
        var mouse2 = item.mapFromItem(source,
                                      connector.x + connector.width / 2,
                                      connector.y + connector.height / 2)
        var tolerance = connector.width / 2 + item.width * 0.1
        if(mouse2.x > -tolerance && mouse2.x < item.width + tolerance
                && mouse2.y > -tolerance && mouse2.y < item.height + tolerance) {
            return true
        } else {
            return false
        }
    }

    function compartmentUnderConnector(source, connector) {
        var compartment = undefined
        for(var i in compartments) {
            var targetCompartment = compartments[i]
            if(isItemUnderConnector(targetCompartment, source, connector)) {
                compartment = targetCompartment
            }
        }
        return compartment
    }

    function voltmeterUnderConnector(source, connector) {
        var voltmeter = undefined
        for(var i in voltmeters) {
            var targetVoltmeter = voltmeters[i]
            if(isItemUnderConnector(targetVoltmeter, source, connector)) {
                voltmeter = targetVoltmeter
            }
        }
        return voltmeter
    }

    function deselectAll() {
        deselectCompartmentConnections()
        deselectCompartments()
        deselectVoltmeters()
        deselectVoltmeterConnections()
    }

    function deselectAllInList(listName) {
        for(var i in listName) {
            var listObject = listName[i]
            listObject.selected = false
        }
    }

    function deselectVoltmeterConnections() {
        connectionControls.connection = null
        deselectAllInList(voltmeterConnections)
    }

    function deselectCompartmentConnections() {
        connectionControls.connection = null
        deselectAllInList(compartmentConnections)
    }

    function deselectCompartments() {
        compartmentControls.compartment = null
        deselectAllInList(compartments)
    }

    function deselectVoltmeters() {
        voltmeterControls.voltmeter = null
        deselectAllInList(voltmeters)
    }

    function clickedCompartment(compartment) {
        deselectAll()
        compartmentControls.compartment = compartment
        compartment.selected = true
    }

    function clickedConnection(connection) {
        deselectAll()
        connectionControls.connection = connection
        connection.selected = true
    }

    function clickedVoltmeter(voltmeter) {
        deselectAll()
        voltmeterControls.voltmeter = voltmeter
        voltmeter.selected = true
    }

    function createCompartment(properties) {
        var component = Qt.createComponent("Compartment.qml")
        var compartment = component.createObject(compartmentLayer, properties)
        compartment.dragStarted.connect(resetOrganize)
        compartment.clicked.connect(clickedCompartment)
        compartment.droppedConnectionCreator.connect(createConnectionToPoint)
        compartments.push(compartment)
        return compartment
    }

    function createVoltmeter(properties) {
        var component = Qt.createComponent("Voltmeter.qml")
        var voltmeter = component.createObject(compartmentLayer, properties)
        voltmeters.push(voltmeter)
        voltmeter.clicked.connect(clickedVoltmeter)
        return voltmeter
    }

    function createConnection(sourceObject, targetObject) {
        var connectionComponent = Qt.createComponent("Connection.qml")
        var connection = connectionComponent.createObject(connectionLayer, {
                                                              sourceCompartment: sourceObject,
                                                              targetCompartment: targetObject
                                                          })
        connection.clicked.connect(clickedConnection)
        return connection
    }

    function connectCompartments(sourceCompartment, targetCompartment) {
        var connection = createConnection(sourceCompartment, targetCompartment)
        sourceCompartment.connections.push(connection)
        targetCompartment.connections.push(connection)
        compartmentConnections.push(connection)
        return connection
    }

    function connectVoltmeter(compartment, voltmeter) {
        var connection = createConnection(compartment, voltmeter)
        voltmeter.addConnection(connection)
        voltmeterConnections.push(connection)
    }

    function createConnectionToPoint(sourceCompartment, connectionCreator) {
        var targetVoltmeter = voltmeterUnderConnector(sourceCompartment, connectionCreator)
        if(targetVoltmeter) {
            var connectionAlreadyExists = false
            for(var j in voltmeterConnections) {
                var existingConnection = voltmeterConnections[j]
                if((existingConnection.sourceCompartment === sourceCompartment && existingConnection.targetCompartment === targetVoltmeter)
                        || (existingConnection.targetCompartment === sourceCompartment && existingConnection.sourceCompartment === targetVoltmeter)) {
                    connectionAlreadyExists = true
                    break
                }
            }
            if(connectionAlreadyExists) {
                return
            }
            connectVoltmeter(sourceCompartment, targetVoltmeter)
            return
        }

        var targetCompartment = compartmentUnderConnector(sourceCompartment, connectionCreator)
        if(!targetCompartment || targetCompartment === sourceCompartment) {
            return
        }
        var connectionAlreadyExists = false
        for(var j in compartmentConnections) {
            var existingConnection = compartmentConnections[j]
            if((existingConnection.sourceCompartment === sourceCompartment && existingConnection.targetCompartment === targetCompartment)
                    || (existingConnection.targetCompartment === sourceCompartment && existingConnection.sourceCompartment === targetCompartment)) {
                connectionAlreadyExists = true
                break
            }
        }
        if(connectionAlreadyExists) {
            return
        }
        connectCompartments(sourceCompartment, targetCompartment)
        resetOrganize()
    }

    function resetOrganize() {
        lastOrganizeTime = Date.now()
        layoutTimer.start()
    }

    function organize() {
        var currentOrganizeTime = Date.now()
        var dt = Math.min(0.032, (currentOrganizeTime - lastOrganizeTime) / 1000.0)
        var springLength = simulatorRoot.width * 0.1
        var anyDragging = false

        for(var i in compartments) {
            var compartment = compartments[i]
            compartment.velocity = Qt.vector2d(0,0)
            if(compartment.dragging) {
                anyDragging = true
            }
        }

        for(var i in compartmentConnections) {
            var connection = compartmentConnections[i]
            var source = connection.sourceCompartment
            var target = connection.targetCompartment
            var xDiff = source.x - target.x
            var yDiff = source.y - target.y
            var length = Math.sqrt(xDiff*xDiff + yDiff*yDiff)
            var lengthDiff = length - springLength
            var xDelta = lengthDiff * xDiff / length
            var yDelta = lengthDiff * yDiff / length
            var kFactor = lengthDiff > 0 ? 0.015 : 0.007
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
                var k = simulatorRoot.width * 0.005
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
            layoutTimer.stop()
        }

        lastOrganizeTime = currentOrganizeTime
    }

    MouseArea {
        anchors.fill: parent
        onClicked: {
            deselectAll()
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

    CreationControls {
        onCreateCompartment: {
            simulatorRoot.createCompartment(position)
        }
        onCreateVoltmeter: {
            simulatorRoot.createVoltmeter(position)
        }
    }

    CompartmentControls {
        id: compartmentControls
        onDisconnectClicked: {
            simulatorRoot.disconnectCompartment(compartment)
        }
        onDeleteClicked: {
            simulatorRoot.deleteCompartment(compartment)
        }
    }

    VoltmeterControls {
        id: voltmeterControls
    }

    ConnectionControls {
        id: connectionControls
    }

    Rectangle {
        id: playbackControls

        anchors {
            bottom: parent.bottom
            left: parent.left
        }

        color: "#deebf7"
        border.color: "#9ecae1"
        border.width: 1.0

        width: parent.width / 2.0
        height: parent.height * 0.08

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
                property real realValue: Math.pow(10, value)
                minimumValue: -1
                maximumValue: 1
            }

            Text {
                text: playbackSpeedSlider.realValue.toFixed(1) + " x"
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

            Item {
                Layout.fillHeight: true
                Layout.fillWidth: true
            }
        }
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
        running: playingCheckbox.checked
        repeat: true
        onRunningChanged: {
            if(running) {
                lastStepTime = Date.now()
            }
        }

        onTriggered: {
            var currentTime = Date.now()
            var dt = (currentTime - lastStepTime) / 1000
            dt *= playbackSpeedSlider.realValue
            for(var i in compartments) {
                var compartment = compartments[i]
                compartment.stepForward(dt)
            }
            for(var i in compartments) {
                var compartment = compartments[i]
                compartment.finalizeStep(dt)
            }
            for(var i in voltmeters) {
                var voltmeter = voltmeters[i]
                voltmeter.stepForward(dt)
            }
            lastStepTime = currentTime
        }
    }
}
