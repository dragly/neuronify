import QtQuick 2.0
import QtQuick.Controls 1.2
import QtQuick.Layouts 1.1
import "hud"

Rectangle {
    id: simulatorRoot

    property real lastOrganizeTime: Date.now()
    property real lastStepTime: Date.now()
    property var connections: [] // List used to deselect all connections
    property var organizedItems: []
    property var organizedConnections: []
    property var compartments: []
    property var synapses: []
    property var voltmeters: []
    //    property var connections: []

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
        var synapse = createSynapse({x: 600, y: 100})
//        connectSynapse(previousCompartment, synapse)
        connectVoltmeter(synapse.postSynapse, voltmeter)
    }

    function deleteFromList(list, item) {
        var itemIndex = list.indexOf(item)
        if(itemIndex > -1) {
            list.splice(itemIndex, 1)
        }
    }

    function deleteCompartment(compartment) {
        disconnectCompartment(compartment)
        if(compartmentControls.compartment === compartment) {
            compartmentControls.compartment = null
        }
        deleteFromList(compartments, compartment)
        deleteFromList(organizedItems, compartment)
        compartment.destroy()
        resetOrganize()
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
        resetOrganize()
    }

    function disconnectVoltmeter(voltmeter) {
        var connectionsToDelete = voltmeter.connectionPlots
        for(var i in connectionsToDelete) {
            var connection = connectionsToDelete[i]
            deleteConnection(connection.connection)
        }
        resetOrganize()
    }

    function deleteConnection(connection) {
        deleteFromList(organizedConnections, connection)
        deleteFromList(connections, connection)
        connection.itemA.removeConnection(connection)
        connection.itemB.removeConnection(connection)
        connection.destroy()
        resetOrganize()
    }

    function disconnectCompartment(compartment) {
        var connectionsToDelete = compartment.connections.concat(compartment.passiveConnections)
        compartment.connections = []
        for(var i in connectionsToDelete) {
            var connection = connectionsToDelete[i]
            deleteConnection(connection)
        }
        resetOrganize()
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

    function itemUnderConnector(itemList, source, connector) {
        var item = undefined
        for(var i in itemList) {
            var itemB = itemList[i]
            if(isItemUnderConnector(itemB, source, connector)) {
                item = itemB
            }
        }
        return item
    }

    function deselectAll() {
        deselectConnections()
        deselectCompartments()
        deselectVoltmeters()
    }

    function deselectAllInList(listName) {
        for(var i in listName) {
            var listObject = listName[i]
            listObject.selected = false
        }
    }

    function deselectConnections() {
        connectionControls.connection = null
        deselectAllInList(connections)
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
        compartment.droppedConnector.connect(createConnectionToPoint)
        compartments.push(compartment)
        organizedItems.push(compartment)
        resetOrganize()
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
        //        synapse.droppedConnector.connect(createConnectionToPoint)
        synapses.push(synapse)
        organizedItems.push(synapse)
        resetOrganize()
        return synapse
    }

    function createVoltmeter(properties) {
        var component = Qt.createComponent("Voltmeter.qml")
        var voltmeter = component.createObject(compartmentLayer, properties)
        voltmeter.x = Math.max(voltmeter.x, creationControls.width)
        voltmeters.push(voltmeter)
        voltmeter.clicked.connect(clickedVoltmeter)
        resetOrganize()
        return voltmeter
    }

    function createConnection(sourceObject, targetObject) {
        var connectionComponent = Qt.createComponent("Connection.qml")
        var connection = connectionComponent.createObject(connectionLayer, {
                                                              itemA: sourceObject,
                                                              itemB: targetObject
                                                          })
        connection.clicked.connect(clickedConnection)
        return connection
    }

    function connectCompartments(itemA, itemB) {
        var connection = createConnection(itemA, itemB)
        itemA.addConnection(connection)
        itemB.addConnection(connection)
        organizedConnections.push(connection)
        connections.push(connection)
        return connection
    }

    function connectVoltmeter(compartment, voltmeter) {
        var connection = createConnection(compartment, voltmeter)
        voltmeter.addConnection(connection)
        compartment.addPassiveConnection(connection)
        connections.push(connection)
    }

    function connectSynapse(compartment, synapse, connector) {
        var compartmentUnderConnector = synapse.compartmentUnderConnector(compartment.mapToItem(synapse, connector.x, connector.y))
        var connection = createConnection(compartment, compartmentUnderConnector)
        compartment.addConnection(connection)
        compartmentUnderConnector.addConnection(connection)
        connections.push(connection)
    }

    function connectionExists(itemA, itemB) {
        var connectionAlreadyExists = false
        for(var j in connections) {
            var existingConnection = connections[j]
            if((existingConnection.itemA === itemA && existingConnection.itemB === itemB)
                    || (existingConnection.itemB === itemB && existingConnection.itemA === itemA)) {
                connectionAlreadyExists = true
                break
            }
        }
        return connectionAlreadyExists
    }

    function createConnectionToPoint(itemA, connector) {
        var targetSynapse = itemUnderConnector(synapses, itemA, connector)
        if(targetSynapse) {
            if(!connectionExists(itemA, targetSynapse)) {
                connectSynapse(itemA, targetSynapse, connector)
                return
            }
        }

        var targetVoltmeter = itemUnderConnector(voltmeters, itemA, connector)
        if(targetVoltmeter) {
            if(!connectionExists(itemA, targetVoltmeter)) {
                connectVoltmeter(itemA, targetVoltmeter)
                return
            }
        }

        var itemB = itemUnderConnector(compartments, itemA, connector)
        if(!itemB) {
            return
        }
        if(itemB === itemA) {
            return
        }
        if(connectionExists(itemA, itemB)) {
            return
        }
        connectCompartments(itemA, itemB)
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
        var springLength = simulatorRoot.width * 0.04
        var anyDragging = false

        for(var i in organizedItems) {
            var item = organizedItems[i]
            item.velocity = Qt.vector2d(0,0)
            if(item.dragging) {
                anyDragging = true
            }
        }

        for(var i in organizedConnections) {
            var connection = organizedConnections[i]
            var source = connection.itemA
            var target = connection.itemB
            var totalSpringLength = source.width / 2.0 + target.width / 2.0 + springLength
            var sourceCenter = compartmentCenter(source)
            var targetCenter = compartmentCenter(target)
            var xDiff = sourceCenter.x - targetCenter.x
            var yDiff = sourceCenter.y - targetCenter.y
            var length = Math.sqrt(xDiff*xDiff + yDiff*yDiff)
            var lengthDiff = length - totalSpringLength
            var xDelta = lengthDiff * xDiff / length
            var yDelta = lengthDiff * yDiff / length
            var kFactor = lengthDiff > 0 ? 0.015 : 0.005
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

        for(var i = 0; i < organizedItems.length; i++) {
            var minDistance = 50
            var guard = 1.0
            var compartmentA = organizedItems[i]
            for(var j = i + 1; j < organizedItems.length; j++) {
                var compartmentB = organizedItems[j]
                var totalMinDistance = Math.max(compartmentA.height, compartmentA.width) / 2.0
                        + Math.max(compartmentB.height, compartmentB.width) / 2.0
                        + minDistance
                var centerA = compartmentCenter(compartmentA)
                var centerB = compartmentCenter(compartmentB)
                var xDiff = centerA.x - centerB.x
                var yDiff = centerA.y - centerB.y
                var length = Math.sqrt(xDiff*xDiff + yDiff*yDiff)
                if(length < guard) {
                    continue
                }
                var lengthDiff = length - totalMinDistance
                if(lengthDiff > 0.0) {
                    continue
                }

                var xDelta = lengthDiff * xDiff / length
                var yDelta = lengthDiff * yDiff / length
                var k = simulatorRoot.width * 0.007
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
        for(var i in organizedItems) {
            var item = organizedItems[i]
            var speed = Math.sqrt(item.velocity.x*item.velocity.x + item.velocity.y*item.velocity.y)
            if(speed > maxSpeed && speed > 0) {
                item.velocity.x *= (maxSpeed / speed)
                item.velocity.y *= (maxSpeed / speed)
            }

            maxAppliedSpeed = Math.max(maxAppliedSpeed, item.velocity.x*item.velocity.x + item.velocity.y*item.velocity.y)
            item.x += item.velocity.x * dt
            item.y += item.velocity.y * dt

            item.x = Math.max(item.x, creationControls.width - item.width * 0.5)
            item.y = Math.max(item.y,  - item.height * 0.5)
            item.x = Math.min(item.x, compartmentLayer.width - item.width * 0.5)
            item.y = Math.min(item.y, compartmentLayer.height - playbackControls.height - item.height  * 0.5)
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
            for(var i in synapses) {
                var synapse = synapses[i]
                synapse.stepForward(dt)
            }
            for(var i in compartments) {
                var compartment = compartments[i]
                compartment.finalizeStep()
            }
            for(var i in synapses) {
                var synapse = synapses[i]
                synapse.finalizeStep()
            }
            for(var i in voltmeters) {
                var voltmeter = voltmeters[i]
                voltmeter.stepForward(dt)
            }
            lastStepTime = currentTime
        }
    }
}
