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
import "tools"

Rectangle {
    id: neuronifyRoot

    property real lastStepTime: Date.now()
    property var connections: [] // List used to deselect all connections
    property var organizedItems: []
    property var organizedConnections: []
    property var entities: []
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
        fileManager.saveState(fileUrl)
    }

    function loadState(fileUrl) {
        deleteEverything()
        creationControls.autoLayout = false
        var code = fileManager.read(fileUrl)
        eval(code)
    }

    function deleteEverything() {
        console.log("Deleting everything")
        var connectionsToDelete = connections.slice()
        for(var i in connectionsToDelete) {
            connectionsToDelete[i].destroy()
        }

        var entitiesToDelete = entities.slice()
        for(var i in entitiesToDelete) {
            entitiesToDelete[i].destroy()
        }
    }

    function cleanupDeletedEntity(entity) {
        if(selectedEntities.indexOf(entity) !== -1) {
            deselectAll()
        }
        deleteFromList(autoLayout.entities, entity)
        deleteFromList(voltmeters, entity)
        deleteFromList(entities, entity)

        for(var i in connections) {
            var connection = connections[i]
            if(connection.itemA === entity || connection.itemB === entity) {
                connection.destroy()
            }
        }

        resetOrganize()
    }

    function cleanupDeletedConnection(connection) {
        deleteFromList(autoLayout.connections, connection)
        deleteFromList(connections, connection)
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

    function itemUnderConnector(source, connector, callback) {
        var item = undefined
        for(var i in entities) {
            var itemB = entities[i]
            if(isItemUnderConnector(itemB, source, connector)) {
                item = itemB
            }
        }
        return item
    }

    function selectAll() {
        for(var i in entities) {
            var listObject = entities[i]
            listObject.selected = true
            selectedEntities.push(listObject)
        }
    }

    function deselectAll() {
        activeObject = null
        for(var i in entities) {
            var listObject = entities[i]
            listObject.selected = false
        }
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

        for(var i in selectedEntities) {
            var selectedEntity = selectedEntities[i]
            selectedEntity.selected = true
        }
    }

    function clickedConnection(connection) {
        deselectAll()
        activeObject = connection
        connection.selected = true
    }

    function createEntity(fileUrl, properties, useAutoLayout) {
        var component = Qt.createComponent(fileUrl)
        properties.simulator = neuronifyRoot
        var entity = component.createObject(neuronLayer, properties)
        entity.dragStarted.connect(resetOrganize)
        entity.widthChanged.connect(resetOrganize)
        entity.heightChanged.connect(resetOrganize)
        entity.clicked.connect(clickedEntity)
        entity.aboutToDie.connect(cleanupDeletedEntity)
        entities.push(entity)
        if(useAutoLayout) {
            autoLayout.entities.push(entity)
            resetOrganize()
        }
        return entity
    }

    function createNeuron(properties) {
        console.warn("Using deprecated createNeuron function! Please update your scripts or savefiles.")
        var neuron = createEntity("Neuron.qml", properties, true)
        return neuron
    }

    function createTouchSensor(properties) {
        console.warn("Using deprecated createTouchSensor function! Please update your scripts or savefiles.")
//        properties.dropFunction = createConnectionToPoint
        var sensor = createEntity("TouchSensor.qml", properties)
        return sensor
    }

    function createVoltmeter(properties) {
        console.warn("Using deprecated createVoltmeter function! Please update your scripts or savefiles.")
        var voltmeter = createEntity("Voltmeter.qml", properties)
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

    function connectEntities(itemA, itemB) {
        var connection = createConnection(itemA, itemB)
        autoLayout.connections.push(connection)
        connections.push(connection)
        connection.aboutToDie.connect(cleanupDeletedConnection)
        resetOrganize()
        return connection
    }

    function connectNeurons(itemA, itemB) {
        console.warn("Using deprecated connectNeurons function! Please update your scripts or savefiles.")
        return connectEntities(itemA, itemB)
    }

    function connectSensorToNeuron(sensor, neuron) {
        console.warn("Using deprecated connectSensorToNeuron function! Please update your scripts or savefiles.")
        var connection = createConnection(sensor, neuron)
        sensor.addConnection(connection)
        connections.push(connection)
        return connection
    }

    function connectVoltmeterToNeuron(neuron, voltmeter) {
        console.warn("Using deprecated connectVoltmeterToNeuron function! Please update your scripts or savefiles.")
        return connectEntities(neuron, voltmeter)
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
        var targetEntity = itemUnderConnector(itemA, connector)
        if(targetEntity) {
            if(connectionExists(itemA, targetEntity)) {
                return
            }
            if(itemA === targetEntity) {
                return
            }
            connectEntities(itemA, targetEntity)
            return
        }
    }

    function resetOrganize() {
        autoLayout.resetOrganize()
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

    AutoLayout {
        id: autoLayout
        enabled: creationControls.autoLayout
        maximumWidth: neuronLayer.width
        maximumHeight: neuronLayer.height
    }

    Clipboard {
        id: clipboard
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

        onDroppedEntity: {
            var workspacePosition = creationControls.mapToItem(neuronLayer, position.x, position.y)
            neuronifyRoot.createEntity(fileUrl, workspacePosition, useAutoLayout)
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
            fileManager.showSaveDialog()
            mainMenu.revealed = false
        }

        onLoadSimulationRequested: {
            fileManager.showLoadDialog()
            mainMenu.revealed = false
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

            for(var i in entities) {
                var entity = entities[i]
                entity.step(dt)
            }

            for(var i in connections) {
                var connection = connections[i]
                var itemA = connection.itemA
                var itemB = connection.itemB

                if(connection.valid) {
                    itemA.outputConnectionStep(itemB)
                    itemB.inputConnectionStep(itemA)
                }
            }

            for(var i in entities) {
                var entity = entities[i]
                entity.finalizeStep(dt)
            }

            lastStepTime = currentTime
        }
    }

    FileManager {
        id: fileManager

        entities: neuronifyRoot.entities
        connections: neuronifyRoot.connections

        onLoadState: {
            neuronifyRoot.loadState(fileUrl)
        }
    }

    //////////////////////// save/load ////////////////

    Keys.onPressed: {
        if(event.modifiers & Qt.ControlModifier && event.key=== Qt.Key_A){
            selectAll()
        }
        if(event.modifiers & Qt.ControlModifier && event.key=== Qt.Key_C){
            clipboard.copyNeurons()
        }
        if(event.modifiers & Qt.ControlModifier && event.key=== Qt.Key_V){
            clipboard.pasteNeurons()
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
