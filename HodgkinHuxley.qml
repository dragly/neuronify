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
    property var synapses: []
    property var voltmeterConnections: []
    property var compartmentConnections: []

    width: 400
    height: 300
    color: "#deebf7"
    antialiasing: true
    smooth: true

    Component.onCompleted: {
        var previousCompartment = undefined
        var previousCompartment2 = undefined
        for(var i = 0; i < 5; i++) {
            var compartment = createCompartment({x: 300 + i * 100, y: 200 + (Math.random()) * 10})
            if(previousCompartment) {
                connectCompartments(previousCompartment, compartment)
            }
            if(i === 0) {
                compartment.targetVoltage = -6.0
                compartment.forceTargetVoltage = true
            }
            if(i === 1 || i === 4) {
                var voltmeter = createVoltmeter({x: 300 + i * 100, y: 400})
                connectVoltmeter(compartment, voltmeter)
            }
            previousCompartment = compartment
        }
        createSynapse({x: 200, y: 100})
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
        deselectSynapses()
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

    function deselectSynapses() {
        //        compartmentControls.compartment = null
        deselectAllInList(synapses)
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

    function clickedSynapse(synapse) {
        deselectAll()
        //        compartmentControls.compartment = compartment
        synapse.selected = true
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
        compartment.x = Math.max(compartment.x, creationControls.width)
        compartment.dragStarted.connect(resetOrganize)
        compartment.widthChanged.connect(resetOrganize)
        compartment.heightChanged.connect(resetOrganize)
        compartment.clicked.connect(clickedCompartment)
        compartment.droppedConnectionCreator.connect(createConnectionToPoint)
        compartments.push(compartment)
        return compartment
    }

    function createSynapse(properties) {
        var component = Qt.createComponent("Synapse.qml")
        var synapse = component.createObject(compartmentLayer, properties)
        synapse.x = Math.max(synapse.x, creationControls.width)
        synapse.dragStarted.connect(resetOrganize)
        synapse.widthChanged.connect(resetOrganize)
        synapse.heightChanged.connect(resetOrganize)
        synapse.clicked.connect(clickedSynapse)
        synapse.droppedConnectionCreator.connect(createConnectionToPoint)
        synapses.push(synapse)
        return synapse
    }

    function createVoltmeter(properties) {
        var component = Qt.createComponent("Voltmeter.qml")
        var voltmeter = component.createObject(compartmentLayer, properties)
        voltmeter.x = Math.max(voltmeter.x, creationControls.width)
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

    function compartmentCenter(compartment) {
        return Qt.vector2d(compartment.x + compartment.width / 2, compartment.y + compartment.height / 2)
    }

    function organize() {
        var currentOrganizeTime = Date.now()
        var dt = Math.min(0.032, (currentOrganizeTime - lastOrganizeTime) / 1000.0)
        var springLength = simulatorRoot.width * 0.03
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
            var totalSpringLength = source.width / 2.0 + target.width / 2.0 + springLength
            var sourceCenter = compartmentCenter(source)
            var targetCenter = compartmentCenter(target)
            var xDiff = sourceCenter.x - targetCenter.x
            var yDiff = sourceCenter.y - targetCenter.y
            var length = Math.sqrt(xDiff*xDiff + yDiff*yDiff)
            var lengthDiff = length - totalSpringLength
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
            var guard = 1.0
            var compartmentA = compartments[i]
            for(var j = i + 1; j < compartments.length; j++) {
                var compartmentB = compartments[j]
                var totalMinDistance = source.width / 2.0 + target.width / 2.0 + minDistance
                var centerA = compartmentCenter(compartmentA)
                var centerB = compartmentCenter(compartmentB)
                var xDiff = centerA.x - centerB.x
                var yDiff = centerA.y - centerB.y
                var length = Math.sqrt(xDiff*xDiff + yDiff*yDiff)
                if(length > minDistance) {
                    continue
                }
                if(length < guard) {
                    continue
                }

                var lengthDiff = length - totalMinDistance
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

        var maxAppliedSpeed = 0.0
        var maxSpeed = simulatorRoot.width * 1.0
        var minSpeed = simulatorRoot.width * 0.5
        for(var i in compartments) {
            var compartment = compartments[i]
            var speed = Math.sqrt(compartment.velocity.x*compartment.velocity.x + compartment.velocity.y*compartment.velocity.y)
            if(speed > maxSpeed && speed > 0) {
                compartment.velocity.x *= (maxSpeed / speed)
                compartment.velocity.y *= (maxSpeed / speed)
            }

            maxAppliedSpeed = Math.max(maxAppliedSpeed, compartment.velocity.x*compartment.velocity.x + compartment.velocity.y*compartment.velocity.y)
            compartment.x += compartment.velocity.x * dt
            compartment.y += compartment.velocity.y * dt

            compartment.x = Math.max(compartment.x, creationControls.width - compartment.width * 0.5)
            compartment.y = Math.max(compartment.y,  - compartment.height * 0.5)
            compartment.x = Math.min(compartment.x, compartmentLayer.width - compartment.width * 0.5)
            compartment.y = Math.min(compartment.y, compartmentLayer.height - playbackControls.height - compartment.height  * 0.5)
        }

        if(maxAppliedSpeed < minSpeed && !anyDragging) {
            layoutTimer.stop()
        }

        lastOrganizeTime = currentOrganizeTime
    }

    Item {
        id: workspaceFlickable
        anchors.fill: parent
//        contentWidth: 3840 // * workspace.scale
//        contentHeight: 2160 // * workspace.scale

        MouseArea {
            id: workspaceMouseArea
            anchors.fill: parent

            drag.target: workspace

            property vector2d last
            property vector2d image

            onWheel: {
                var relativeMouse = mapToItem(workspace, wheel.x, wheel.y)
                workspaceScale.origin.x = relativeMouse.x
                workspaceScale.origin.y = relativeMouse.y
                workspaceScale.xScale = Math.min(1.0, Math.max(0.1, workspaceScale.xScale + wheel.angleDelta.y * 0.001))
                var newPosition = mapFromItem(workspace, relativeMouse.x, relativeMouse.y)
                workspace.x += wheel.x - newPosition.x
                workspace.y += wheel.y - newPosition.y
            }

            onClicked: {
//                workspaceScale.origin.x = mouse.x
//                workspaceScale.origin.y = mouse.y
                    deselectAll()
            }
        }

        Item {
            id: workspace

            width: 3840
            height: 2160

            transform: Scale {
                id: workspaceScale
                yScale: xScale
            }

            Rectangle {
                anchors.fill: parent
                color: "#f7fbff"
            }

            Item {
                id: connectionLayer
                anchors.fill: parent
            }

            Item {
                id: compartmentLayer
                anchors.fill: parent
            }
        }

    }

    CreationControls {
        id: creationControls
        onCreateCompartment: {
            var workspacePosition = simulatorRoot.mapToItem(compartmentLayer, position.x, position.y)
            simulatorRoot.createCompartment(workspacePosition)
        }
        onCreateVoltmeter: {
            var workspacePosition = simulatorRoot.mapToItem(compartmentLayer, position.x, position.y)
            simulatorRoot.createVoltmeter(workspacePosition)
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
        onDeleteClicked: {
            simulatorRoot.deleteConnection(connection)
        }
    }

    Rectangle {
        id: playbackControls

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
                property real realValue: Math.pow(10, value)
                minimumValue: -1
                maximumValue: 1.2
                Layout.fillWidth: true
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
            dt *= 3.0
            dt *= playbackSpeedSlider.realValue
            dt = Math.min(0.050, dt)
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
