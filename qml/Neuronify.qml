import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Dialogs 1.0
import QtQuick.Window 2.1
import QtCharts 2.1
import QtMultimedia 5.5

import Neuronify 1.0

import "hud"
import "menus/mainmenu"
import "style"
import "io"
import "tools"

/*!
  \qmltype Neuronify
  \inqmlmodule Neuronify
  \ingroup neuronify
  \brief This type holds the application.

  This item is created in the \l{ApplicationWindow}. It contains all the menues
  as well as the game canvas.
*/

Rectangle {
    id: root

    property var organizedItems: []
    property var organizedConnections: []
    property alias graphEngine: graphEngine
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
    property bool running: applicationActive && !mainMenu.revealed
    property string clickMode: "selection"

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

    color: "#deebf7"
    antialiasing: true
    smooth: true
    focus: true

    Component.onCompleted: {
        var latest = StandardPaths.locate(StandardPaths.AppConfigLocation, "latest.nfy")
        if(latest !== "") {
            loadState("file://" + StandardPaths.writableLocation(StandardPaths.AppConfigLocation) + "/latest.nfy")
        } else {
            loadState("qrc:/simulations/singleCell/singleCell.nfy")
        }
        resetStyle()
    }

    Component.onDestruction: {
        saveState("file://" + StandardPaths.writableLocation(StandardPaths.AppConfigLocation) + "/latest.nfy")
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
        for(var i in graphEngine.nodes) {
            var entity = graphEngine.nodes[i]
            fileString += entity.dump(i)
        }

        for(var i in graphEngine.edges) {
            var connection = graphEngine.edges[i]
            fileString += connection.dump(i, graphEngine)
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
        var connectionsToDelete = graphEngine.edges
        for(var i in connectionsToDelete) {
            connectionsToDelete[i].destroy(1)
        }
        graphEngine.edges = []
        var entitiesToDelete = graphEngine.nodes; //.slice()
        for(var i in entitiesToDelete) {
            entitiesToDelete[i].destroy(1)
        }
        graphEngine.nodes = []
    }

    function cleanupDeletedEntity(entity) {
        if(selectedEntities.indexOf(entity) !== -1) {
            deselectAll()
        }
        // TODO should these be removed
        deleteFromList(autoLayout.entities, entity)
        deleteFromList(voltmeters, entity)

        resetOrganize()
    }

    function cleanupDeletedConnection(connection) {
        deleteFromList(autoLayout.connections, connection)
        graphEngine.removeEdge(connection)
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
        for(var i in graphEngine.nodes) {
            var itemB = graphEngine.nodes[i]
            if(isItemUnderConnector(itemB, source, connector)) {
                item = itemB
            }
        }
        return item
    }

    function selectAll() {
        for(var i in graphEngine.nodes) {
            var listObject = graphEngine.nodes[i]
            listObject.selected = true
            selectedEntities.push(listObject)
        }
    }

    function deselectAll() {
        clickMode = "selection"
        selectedEntities.length = 0
        activeObject = null
        for(var i in graphEngine.nodes) {
            var listObject = graphEngine.nodes[i]
            listObject.selected = false
        }
        for(var i in graphEngine.edges) {
            var connection = graphEngine.edges[i]
            connection.selected = false
        }
    }

    function deleteSelected() {
        var toDelete = []
        for(var i in selectedEntities) {
            toDelete.push(selectedEntities[i])
        }
        for(var i in toDelete) {
            var node = toDelete[i]
            graphEngine.removeNode(node)
            node.destroy(1)
        }


        if(activeObject) {
            activeObject.destroy(1)
        }

        deselectAll()
    }

    function clickedEntity(entity, mouse) {
        if(clickMode === "selection") {
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
        } else if (clickMode === "connection") {
            connectEntities(activeObject, entity)
        }
    }

    function clickedConnection(connection) {
        deselectAll()
        activeObject = connection
        connection.selected = true
    }

    function clickedConnector() {
        clickMode = "connection"
    }

    function createEntity(fileUrl, properties, useAutoLayout) {
        var component = Qt.createComponent(fileUrl)
        if(component.status !== Component.Ready) {
            console.error("Could not create component of type " + fileUrl)
            console.error(component.errorString())
            return
        }

        var isRetina = fileUrl.toString().indexOf("Retina.qml")> -1
        if(isRetina){
            camera.retinaCounter += 1
            properties.videoSurface = videoSurface
        }


        properties.simulator = root
        var entity = component.createObject(neuronLayer, properties)
        if(!entity) {
            console.error("Could not create entity from component " + fileUrl)
            return
        }

        if(isRetina){
            entity.aboutToDie.connect(function(dead){
                camera.retinaCounter-=1
            })
        }

        entity.dragStarted.connect(resetOrganize)
        entity.widthChanged.connect(resetOrganize)
        entity.heightChanged.connect(resetOrganize)
        entity.clicked.connect(clickedEntity)
        entity.aboutToDie.connect(cleanupDeletedEntity)
        entity.clickedConnector.connect(clickedConnector)
        entity.droppedConnector.connect(createConnectionToPoint)

        graphEngine.addNode(entity)
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
        if(itemA === itemB) {
            return
        }
        if(connectionExists(itemA, itemB)) {
            return
        }
        var connection = createConnection(itemA, itemB)
        autoLayout.connections.push(connection)
        graphEngine.addEdge(connection)
        connection.aboutToDie.connect(cleanupDeletedConnection)
        resetOrganize()
        return connection
    }

    function connectionExists(itemA, itemB) {
        var connectionAlreadyExists = false
        for(var j in graphEngine.edges) {
            var existingConnection = graphEngine.edges[j]
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
            connectEntities(itemA, targetEntity)
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

    GraphEngine {
        id: graphEngine
    }

    AutoLayout {
        id: autoLayout
        enabled: false
        //        enabled: creationControls.autoLayout
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

                propagateComposedEvents: true
                drag.target: workspace

                onWheel: {
                    if(wheel.modifiers & Qt.ControlModifier) {
                        var localStartCenter = mapToItem(workspace, wheel.x, wheel.y)
                        var newScale = workspace.scale + wheel.angleDelta.y * 0.001
                        workspace.scale = pinchArea.clampScale(newScale)
                        workspace.x = wheel.x - localStartCenter.x * workspace.scale
                        workspace.y = wheel.y - localStartCenter.y * workspace.scale
                    } else {
                        workspace.x += wheel.angleDelta.x * 0.4
                        workspace.y += wheel.angleDelta.y * 0.4
                    }
                }

                onClicked: {
                    propertiesPanel.revealed = false
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
        onClicked: {
            mainMenu.revealed = true
        }
    }

    CreationMenuButton {
        onClicked: {
            creationMenu.revealed = true
        }
    }


    DeleteButton {
        revealed: activeObject ? true : false
        onClicked: {
            deleteSelected()
        }
    }

    PropertiesButton {
        revealed: activeObject ? true : false
        onClicked: {
            propertiesPanel.revealed = true
        }
    }

    CreationMenu {
        id: creationMenu

        blurSource: workspaceFlickable

        onDroppedEntity: {
            var workspacePosition = controlParent.mapToItem(neuronLayer, properties.x, properties.y)
            properties.x = workspacePosition.x
            properties.y = workspacePosition.y
            root.createEntity(fileUrl, properties, useAutoLayout)
        }

        onDeleteEverything: {
            root.deleteEverything()
        }
    }

    PropertiesPanel {
        id: propertiesPanel
        activeObject: root.activeObject
    }

    Rectangle {
        anchors {
            top: parent.top
            left: parent.left
            right: parent.right
        }
        height: Style.touchableSize
        visible: clickMode === "connection"

        Text {
            anchors.centerIn: parent
            text: "Connection mode: Select other items to connect them."
        }

        Button {
            anchors {
                verticalCenter: parent.verticalCenter
                right: parent.right
            }
            text: "Done"
            onClicked: {
                clickMode = "selection"
            }
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
        id: timer
        interval: 16
        repeat: true
        running: root.running

        onTriggered: {
            var dt = 0.1e-3
            time += dt
            graphEngine.step(dt)
        }
    }

    FileManager {
        id: fileManager

        graphEngine: graphEngine
        otherItems: [workspace]

        onLoadState: {
            console.log("Load state signal caught")
            root.loadState(fileUrl)
        }
    }

    VideoSurface{
        id: videoSurface
        enabled: root.running
        camera: Camera{
            id:camera
            viewfinder.resolution : Qt.size(1280,720)
            property bool active: retinaCounter > 0 && root.running
            property int retinaCounter: 0

            Component.onCompleted: {
                camera.stop()
            }

            onActiveChanged: {
                if(active){
                    camera.start()
                }else{
                    camera.stop()
                }
            }
        }
    }

    VideoOutput {
        // dummy output needed for camera to work on Android
        x: -10
        enabled: Qt.platform.os === "android" && videoSurface.enabled
        visible: Qt.platform.os === "android" && videoSurface.enabled
        width: 10
        height: 10
        source: videoSurface && videoSurface.camera ? videoSurface.camera : null
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
            deleteSelected()
        }
    }

}
