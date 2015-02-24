import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Dialogs 1.0
import QtQuick.Window 2.1

import Neuronify 1.0

import "hud"
import "menus/mainmenu"
import "style"
import "io"

Rectangle {
    id: neuronifyRoot

    property real lastOrganizeTime: Date.now()
    property real lastStepTime: Date.now()
    property var connections: [] // List used to deselect all connections
    property var organizedItems: []
    property var organizedConnections: []
    property var neurons: []
    property var sensors: []
    property var selectedEntities: []
    property var copiedNeurons: []
    property var voltmeters: []
    property real currentTimeStep: 0.0
    property real time: 0.0
    property var activeObject: null
    property bool applicationActive: {
        if(Qt.platform.os === "android" || Qt.platform.os === "ios") {
            if(Qt.application.state === Qt.ApplicationActive) {
                return true
            } else {
                return false
            }
        } else {
            return true
        }
    }

    width: 400
    height: 300
    color: "#deebf7"
    antialiasing: true
    smooth: true
    focus: true

    Component.onCompleted: {
        loadState("/simulations/dummy/dummy.nfy")
        resetStyle()
    }

    function deleteFromList(list, item) {
        var itemIndex = list.indexOf(item)
        if(itemIndex > -1) {
            list.splice(itemIndex, 1)
        }
    }

    function saveState(fileUrl) {
        fileManager.saveState(fileUrl, neurons, voltmeters, sensors)
    }

    function loadState(fileUrl) {
        var code = fileManager.read(fileUrl)
        eval(code)
    }


    //////////////////////// end of save/load ////////////////

    function deleteEverything() {
        deleteAllConnections()
        deleteAllVoltmeters()
        deleteAllNeurons()
        deleteAllSensors()
    }

    function deleteAllNeurons() {
        while(neurons.length > 0){
            deleteNeuron(neurons[0])
        }
    }

    function deleteAllVoltmeters() {
        while(voltmeters.length > 0){
            deleteVoltmeter(voltmeters[0])
        }
    }

    function deleteAllSensors() {
        while(sensors.length > 0){
            deleteSensor(sensors[0])
        }
    }

    function deleteAllConnections() {
        while(connections.length > 0){
            deleteConnection(connections[0])
        }
    }

    function deleteSensor(sensor) {
        deselectAll()
        disconnectSensor(sensor)
        deleteFromList(sensors, sensor)
        sensor.destroy(1)
        resetOrganize()
    }

    function disconnectSensor(sensor) {
        var connectionsToDelete = sensor.connections
        for(var i in connectionsToDelete) {
            var connection = connectionsToDelete[i]
            deleteConnection(connection)
        }
        resetOrganize()
    }

    function deleteNeuron(neuron) {
        console.log("Deleting neuron " + neuron)
        deselectAll()
        deleteFromList(neurons, neuron)
        deleteFromList(organizedItems, neuron)
        resetOrganize()
    }

    function deleteVoltmeter(voltmeter) {
        deleteFromList(voltmeters, voltmeter)
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
        connection.destroy()
        deleteFromList(organizedConnections, connection)
        deleteFromList(connections, connection)
        resetOrganize()
    }

    function disconnectNeuron(neuron) {
        var connectionsToDelete = neuron.connections.concat(neuron.passiveConnections)
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

    function deselectAllInList(listName) {
        for(var i in listName) {
            var listObject = listName[i]
            listObject.selected = false
        }
    }

    function selectAllInList(listName) {
        for(var i in listName) {
            var listObject = listName[i]
            listObject.selected = true
        }
    }

    function deselectPanels() {
        activeObject = null
    }

    function selectAllNeurons() {
        selectedEntities = []
        deselectPanels()
        selectAllInList(neurons)
        for(var i in neurons) {
            var neuron = neurons[i]
            selectedEntities.push(neuron)
        }
    }

    function copyNeurons() {
        copiedNeurons = []
        var copiedNeuron = []
        for(var i in selectedEntities) {
            var neuron = selectedEntities[i]
            copiedNeuron = neuron
            copiedNeurons.push(copiedNeuron)
        }
        selectedEntities = []
    }

    function pasteNeurons() {
        var newNeurons = []
        for(var i in copiedNeurons) {
            var neuronToCopy = copiedNeurons[i]
            var neuron = createNeuron({
                                          x: neuronToCopy.x + 10,
                                          y: neuronToCopy.y + 10,
                                          copiedFrom: neuronToCopy
                                      })

            newNeurons.push(neuron)
        }
        for(var i in copiedNeurons) {
            var oldNeuron = copiedNeurons[i]
            for(var j in newNeurons) {
                var newNeuron = newNeurons[j]
                if(newNeuron.copiedFrom === oldNeuron) {
                    // Find connections
                    for(var k in oldNeuron.connections) {
                        var connectedToNeuron = oldNeuron.connections[k].itemB
                        // Check if connected to copied neuron
                        for(var l in copiedNeurons) {
                            var otherNeuron = copiedNeurons[l]
                            if(otherNeuron === connectedToNeuron) {
                                // Find copied twin
                                for(var m in newNeurons) {
                                    var otherNewNeuron = newNeurons[m]
                                    if(otherNewNeuron.copiedFrom === otherNeuron) {
                                        connectNeurons(newNeuron, otherNewNeuron)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        resetOrganize()
    }

    function selectAll() {
        deselectPanels()
        selectAllInList(connections)
        selectAllInList(voltmeters)
        selectAllInList(neurons)
        selectAllInList(sensors)
    }

    function deselectAll() {
        deselectPanels()
        deselectAllInList(connections)
        deselectAllInList(voltmeters)
        deselectAllInList(neurons)
        deselectAllInList(sensors)
    }

    function clickedEntity(entity, mouse) {
        deselectAll()
        entity.selected = true
        activeObject = entity

        if ((mouse.button === Qt.LeftButton) && (mouse.modifiers & Qt.ShiftModifier)){
            var alreadySelected = false
                for(var j in selectedEntities) {
                    var alreadySelectedEntity = selectedEntities[j]
                    if(alreadySelectedEntity ===  entity) {
                        alreadySelected = true
                    }
                }
                if(!alreadySelected) {
                    selectedEntities.push(entity)
                    console.log(selectedEntities.length)
                }

        } else {
            selectedEntities = []
            selectedEntities.push(entity)
            entity.selected = true
        }

        selectAllInList(selectedEntities)
    }

    function clickedConnection(connection) {
        deselectAll()
        activeObject = connection
        connection.selected = true
    }

    function createNeuron(properties) {
        var component = Qt.createComponent("Neuron.qml")
        var neuron = component.createObject(neuronLayer, properties)
        neuron.dragStarted.connect(resetOrganize)
        neuron.widthChanged.connect(resetOrganize)
        neuron.heightChanged.connect(resetOrganize)
        neuron.clicked.connect(clickedEntity)
        neuron.aboutToDie.connect(deleteNeuron)
        neuron.droppedConnector.connect(createConnectionToPoint)
        neurons.push(neuron)
        organizedItems.push(neuron)
        resetOrganize()
        return neuron
    }

    function createTouchSensor(properties) {
        var component = Qt.createComponent("TouchSensor.qml")
        properties.dropFunction = createConnectionToPoint
        var sensor = component.createObject(neuronLayer, properties)
        sensor.dragStarted.connect(resetOrganize)
        sensor.widthChanged.connect(resetOrganize)
        sensor.heightChanged.connect(resetOrganize)
        sensor.clicked.connect(clickedEntity)
        sensors.push(sensor)
        resetOrganize()
        return sensor
    }

    function createVoltmeter(properties) {
        var component = Qt.createComponent("Voltmeter.qml")
        var voltmeter = component.createObject(neuronLayer, properties)
        voltmeters.push(voltmeter)
        voltmeter.clicked.connect(clickedEntity)
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

    function connectNeurons(itemA, itemB) {
        var connection = createConnection(itemA, itemB)
        itemA.addConnection(connection)
        itemB.addPassiveConnection(connection)
        organizedConnections.push(connection)
        connections.push(connection)
        resetOrganize()
        return connection
    }

    function connectSensorToNeuron(sensor, neuron) {
        console.log("Connecting sensor to neuron")
        var connection = createConnection(sensor, neuron)
        sensor.addConnection(connection)
        neuron.addPassiveConnection(connection)
        connections.push(connection)
        return connection
    }

    function connectVoltmeterToNeuron(neuron, voltmeter) {
        var connection = createConnection(neuron, voltmeter)
        voltmeter.addConnection(connection)
        neuron.addPassiveConnection(connection)
        connections.push(connection)
        return connection
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
        console.log("Creating connection to point")
        var targetVoltmeter = itemUnderConnector(voltmeters, itemA, connector)
        if(targetVoltmeter) {
            if(!connectionExists(itemA, targetVoltmeter)) {
                connectVoltmeterToNeuron(itemA, targetVoltmeter)
                return
            }
        }

        var targetNeuron = itemUnderConnector(neurons, itemA, connector)
        console.log(targetNeuron)
        if(targetNeuron) {
            console.log("Target neuron found")
            if(connectionExists(itemA, targetNeuron)) {
                console.log("Connection already exists")
                return
            }
            if(itemA === targetNeuron) {
                console.log("Cannot connect neuron to itself")
                return
            }
            if(itemA.objectName === "neuron") {
                console.log("Connecting neurons")
                connectNeurons(itemA, targetNeuron)
            }
            if(itemA.objectName === "touchSensorCell") {
                console.log("Connecting sensor to neuron")
                connectSensorToNeuron(itemA, targetNeuron)
            }

            return
        }
    }

    function resetOrganize() {
        lastOrganizeTime = Date.now()
        layoutTimer.start()
    }

    function itemCenter(item) {
        return Qt.vector2d(item.x + item.width / 2, item.y + item.height / 2)
    }

    function organize() {
        if(!creationControls.autoLayout) {
            return
        }

        var currentOrganizeTime = Date.now()
        var dt = Math.min(0.032, (currentOrganizeTime - lastOrganizeTime) / 1000.0)
        var springLength = neuronifyRoot.width * 0.06
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
            var sourceCenter = itemCenter(source)
            var targetCenter = itemCenter(target)
            var xDiff = sourceCenter.x - targetCenter.x
            var yDiff = sourceCenter.y - targetCenter.y
            var length = Math.sqrt(xDiff*xDiff + yDiff*yDiff)
            var lengthDiff = length - totalSpringLength
            var xDelta = lengthDiff * xDiff / length
            var yDelta = lengthDiff * yDiff / length
            var kFactor = lengthDiff > 0 ? 0.015 : 0.005
            var k = kFactor * neuronifyRoot.width
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
            var itemA = organizedItems[i]
            for(var j = i + 1; j < organizedItems.length; j++) {
                var itemB = organizedItems[j]
                var totalMinDistance = Math.max(itemA.height, itemA.width) / 2.0
                        + Math.max(itemB.height, itemB.width) / 2.0
                        + minDistance
                var centerA = itemCenter(itemA)
                var centerB = itemCenter(itemB)
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
                var k = neuronifyRoot.width * 0.007
                if(!itemA.dragging) {
                    itemA.velocity.x -= 0.5 * k * xDelta
                    itemA.velocity.y -= 0.5 * k * yDelta
                }
                if(!itemB.dragging) {
                    itemB.velocity.x += 0.5 * k * xDelta
                    itemB.velocity.y += 0.5 * k * yDelta
                }
            }
        }

        var maxAppliedSpeed = 0.0
        var maxSpeed = neuronifyRoot.width * 1.0
        var minSpeed = neuronifyRoot.width * 0.5
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

            item.x = Math.max(item.x, - item.width * 0.5)
            item.y = Math.max(item.y, - item.height * 0.5)
            item.x = Math.min(item.x, neuronLayer.width - item.width * 0.5)
            item.y = Math.min(item.y, neuronLayer.height - item.height  * 0.5)
        }

        if(maxAppliedSpeed < minSpeed && !anyDragging) {
            layoutTimer.stop()
        }

        lastOrganizeTime = currentOrganizeTime
    }

    function resetStyle() {
        Style.reset(width, height, Screen.pixelDensity)
    }

    onWidthChanged: {
        resetStyle()
    }

    onHeightChanged: {
        resetStyle()
    }

    Item {
        id: workspaceFlickable

        anchors.fill: parent

        PinchArea {
            id: pinchArea
            anchors.fill: parent

            property point workspaceStart
            property var localStartCenter
            property double startScale: 1.0

            function clampScale(scale) {
                return Math.min(3.0, Math.max(0.1, scale))
            }

            onPinchStarted: {
                localStartCenter = mapToItem(workspace, pinch.center.x, pinch.center.y)
                startScale = workspace.scale
            }

            onPinchUpdated: {
                var newScale = pinch.scale * startScale
                workspace.scale = clampScale(newScale)
                workspace.x = pinch.center.x - localStartCenter.x * workspace.scale
                workspace.y = pinch.center.y - localStartCenter.y * workspace.scale
            }

            MouseArea {
                id: workspaceMouseArea
                anchors.fill: parent

                drag.target: workspace

                onWheel: {
                    var localStartCenter = mapToItem(workspace, wheel.x, wheel.y)
                    var newScale = workspace.scale + wheel.angleDelta.y * 0.001
                    workspace.scale = pinchArea.clampScale(newScale)
                    workspace.x = wheel.x - localStartCenter.x * workspace.scale
                    workspace.y = wheel.y - localStartCenter.y * workspace.scale
                }

                onClicked: {
                    deselectAll()
                    selectedEntities = []
                }
            }
        }

        Item {
            id: workspace
            property alias color: workspaceRectangle.color

            width: 3840
            height: 2160

            scale: 1.1
            transformOrigin: Item.TopLeft

//            transform: Scale {
//                id: workspaceScale
//                yScale: xScale
//                xScale: Style.scale
//            }

            Rectangle {
                id: workspaceRectangle
                anchors.fill: parent
                color: "#f7fbff"
            }

            Item {
                id: connectionLayer
                anchors.fill: parent
            }

            Item {
                id: neuronLayer
                anchors.fill: parent
            }
        }

    }

    MainMenuButton {
        revealed: !mainMenu.revealed
    }

    CreationControls {
        id: creationControls
        onCreateNeuron: {
            var workspacePosition = creationControls.mapToItem(neuronLayer, position.x, position.y)
            neuronifyRoot.createNeuron(workspacePosition)
        }

        onCreateVoltmeter: {
            var workspacePosition = creationControls.mapToItem(neuronLayer, position.x, position.y)
            neuronifyRoot.createVoltmeter(workspacePosition)
        }

        onCreateTouchSensor: {
            var workspacePosition = creationControls.mapToItem(neuronLayer, position.x, position.y)
            neuronifyRoot.createTouchSensor(workspacePosition)
        }

        onDeleteEverything: {
            neuronifyRoot.deleteEverything()
        }
    }

    PropertiesPanel {
        id: activeObjectControls
        revealed: activeObject ? true : false
        Loader {
            id: activeObjectControlsLoader
            width: parent.width / 2
            height: parent.height / 2
            sourceComponent: activeObject ? (activeObject.controls ? activeObject.controls : null) : null
        }
    }

    MainMenu {
        id: mainMenu
        anchors.fill: parent
        blurSource: workspaceFlickable

        onLoadSimulation: {
            loadState(simulation.stateSource)
            mainMenu.revealed = false
        }

        onSaveSimulationRequested: {
            saveFileDialog.visible = true
            mainMenu.revealed = false
        }

        onLoadSimulationRequested: {
            loadFileDialog.visible = true
            mainMenu.revealed = false
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
        interval: 16
        running: applicationActive && !mainMenu.revealed
        repeat: true
        onRunningChanged: {
            if(running) {
                lastStepTime = Date.now()
            }
        }

        onTriggered: {
            var currentTime = Date.now()
            var dt = (currentTime - lastStepTime) / 1000
            var trueDt = dt
            dt *= 3.0
            dt = Math.min(0.050, dt)
            currentTimeStep = 0.99 * currentTimeStep + 0.01 * dt
            time += dt
            for(var i in neurons) {
                var neuron = neurons[i]
                neuron.stepForward(dt)
            }
            for(var i in neurons) {
                var neuron = neurons[i]
                neuron.finalizeStep(dt)
            }
            for(var i in voltmeters) {
                var voltmeter = voltmeters[i]
                voltmeter.stepForward(dt)
            }
            for(var i in sensors) {
                var sensor = sensors[i]
                sensor.stepForward(dt)
            }
            lastStepTime = currentTime
        }
    }

    FileManager {
        id: fileManager
        neuronify: neuronifyRoot
    }

    //////////////////////// save/load ////////////////

     Keys.onPressed: {
         if(event.modifiers & Qt.ControlModifier && event.key=== Qt.Key_A){
             selectAllNeurons()
             console.log("select all")
         }
         if(event.modifiers & Qt.ControlModifier && event.key=== Qt.Key_C){
             copyNeurons()
             console.log("copy")
         }
         if(event.modifiers & Qt.ControlModifier && event.key=== Qt.Key_V){
             pasteNeurons()
             console.log("paste")
         }

         if(event.key === Qt.Key_Delete) {
             for(var i in selectedEntities) {
                 var entity = selectedEntities[i]
                 entity.destroy()
             }
             deselectAll()
         }
     }
}
