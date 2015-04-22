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
    property var undoList: [""]
    property int undoIdx: 0
    property int undoIdxCopy: 0
    property bool undoRecordingEnabled: true
    property bool canRedo: false


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
        console.log("Load state called")
        deleteEverything()
        undoList.length = 0

        undoIdx = 1
        undoRecordingEnabled = false
        creationControls.autoLayout = false
        var code = fileManager.read(fileUrl)
        console.log("Evaluating code")
        eval(code)
        undoList.push(code)
        undoRecordingEnabled = true
    }

    function addToUndoList() {
        if (!undoRecordingEnabled){
            return
        }
        var fileString = ""

        var counter = 0
        for(var i in entities) {
            var entity = entities[i]
            fileString += entity.dump(i)
        }

        for(var i in connections) {
            var connection = connections[i]
            fileString += connection.dump(i, entities)
        }

        undoList = undoList.slice(0,undoIdx)
        undoIdx += 1
        undoList.push(fileString)
        console.log("Making new undolist item ", undoIdx, undoList.length)
        canRedo = false
    }

    function undo(){
        if (undoIdx > 1){
            undoIdx -= 1
            deleteEverything()
            console.log("Undoing...", undoIdx, undoList.length)
            undoRecordingEnabled = false
            eval(undoList[undoIdx-1])
            undoRecordingEnabled = true
            canRedo = true
        } else {
            console.log("Nothing to undo! ")
        }
    }

    function redo(){
        if (undoIdx < undoList.length){
            undoIdx += 1
            deleteEverything()

            undoRecordingEnabled = false
            eval(undoList[undoIdx-1])
            undoRecordingEnabled = true
            console.log("Redoing...", undoIdx, undoList.length, undoIdx===undoList.length)
            if (undoIdx === undoList.length){
                canRedo = false
            }
        } else {
            console.log("Something went wrong! ", undoIdx, undoList.length)
        }
    }

    function deleteEverything() {
        console.log("Asked to delete everything")
        var connectionsToDelete = connections.slice()
        for(var i in connectionsToDelete) {
            connectionsToDelete[i].destroy(1)
        }
        connections = []
        var entitiesToDelete = entities.slice()
        for(var i in entitiesToDelete) {
            entitiesToDelete[i].destroy(1)
        }
        entities = []
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
                connection.destroy(1)
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
        selectedEntities.length = 0
        activeObject = null
        for(var i in entities) {
            var listObject = entities[i]
            listObject.selected = false
        }
        for(var i in connections) {
            var connection = connections[i]
            connection.selected = false
        }
    }

    function clickedEntity(entity, mouse) {
        if(activeObject) {
            activeObject.selected = false
        }

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
            }
        } else {
            deselectAll()
            selectedEntities.push(entity)
            entity.selected = true
        }

        for(var i in selectedEntities) {
            var selectedEntity = selectedEntities[i]
            selectedEntity.selected = true
        }

        activeObject = entity
    }

    function clickedConnection(connection) {
        deselectAll()
        activeObject = connection
        connection.selected = true
    }

    function createEntity(fileUrl, properties, useAutoLayout) {
        var component = Qt.createComponent(fileUrl)
        if(component.status !== Component.Ready) {
            console.error("Could not create component of type " + fileUrl)
            console.error(component.errorString())
            return
        }

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
        addToUndoList()
        return entity
    }

    function createConnection(sourceObject, targetObject) {
        var connectionComponent = Qt.createComponent("Connection.qml")
        var connection = connectionComponent.createObject(connectionLayer, {
                                                              itemA: sourceObject,
                                                              itemB: targetObject
                                                          })
        connection.clicked.connect(clickedConnection)
        addToUndoList()
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

            function dump() {
                var properties = ["x", "y", "scale"]
                var output = ""
                for(var i in properties) {
                    output += "workspace." + properties[i] + " = " + workspace[properties[i]] + "\n"
                }
                return output
            }

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
        onRevealedChanged: {
            console.log("Reveal " + revealed)
        }

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
        otherItems: [workspace]

        onLoadState: {
            console.log("Load state signal caught")
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
                entity.destroy(1)
            }
            if(activeObject) {
                activeObject.destroy(1)
            }

            deselectAll()
        }
    }
}
