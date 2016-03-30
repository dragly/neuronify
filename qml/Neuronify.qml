import QtQuick 2.5
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Dialogs 1.0
import QtQuick.Window 2.1
import QtCharts 2.1
import QtMultimedia 5.5
import Qt.labs.settings 1.0

import Neuronify 1.0

import "qrc:/qml/hud"
import "qrc:/qml/menus/mainmenu"
import "qrc:/qml/style"
import "qrc:/qml/io"
import "qrc:/qml/tools"
import "qrc:/qml/controls"

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

    property alias graphEngine: graphEngine
    property var selectedEntities: []
    property var draggedEntity: undefined
    property var copiedNeurons: []
    property real currentTimeStep: 0.0
    property real time: 0.0
    property var activeObject: null
    property var undoList: [""]
    property int undoIdx: 0
    property int undoIdxCopy: 0
    property bool undoRecordingEnabled: true
    property bool canRedo: false
    readonly property bool paused: workspace.playbackSpeed <= 0.0
    readonly property bool running: applicationActive && !mainMenu.revealed && !paused
    property string clickMode: "selection"
    property real highestZ: 0.0
    property bool snappingEnabled: false
    property real snapGridSize: snappingEnabled ? 32.0 : 1.0

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

    color: "#f7fbff"
    antialiasing: true
    smooth: true
    focus: true

    Component.onCompleted: {
        var latest = StandardPaths.locate(StandardPaths.AppConfigLocation, "latest.nfy")
        if(latest !== "") {
            loadSimulation("file://" + StandardPaths.writableLocation(StandardPaths.AppConfigLocation) + "/latest.nfy")
        } else {
            loadSimulation("qrc:/simulations/singleCell/singleCell.nfy")
        }
        resetStyle()
    }

    Component.onDestruction: {
        saveState("file://" + StandardPaths.writableLocation(StandardPaths.AppConfigLocation) + "/latest.nfy")
    }

    Settings {
        property alias snappingEnabled: root.snappingEnabled
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

    function applyProperties(object, properties) {
        if(!object){
            console.warn("WARNING: apply properties got missing object: " + object)
            console.warn("with properties:")
//            for(var i in properties) {
//                console.log(i + ": " + properties[i])
//            }
            console.log("Cannot apply.")
            return
        }

        for(var i in properties) {
            var prop = properties[i];
//            console.log("Setting " + i + ": " + prop)
            if(!object.hasOwnProperty("savedProperties")) {
                console.warn("WARNING: Object " + object + " is missing savedProperties property.");
                continue;
            }
            var found = false;
            for(var j in object.savedProperties) {
                var propertyGroup = object.savedProperties[j];
                if(!propertyGroup.hasOwnProperty(i)) {
                    continue;
                }
                found = true;
                // TODO what if one of them is not an object?
                if(typeof(prop) === "object" && typeof(propertyGroup[i]) == "object") {
                    applyProperties(propertyGroup[i], prop);
                } else {
                    propertyGroup[i] = prop;
                }
            }
            if(!found) {
                console.warn("WARNING: Cannot assign to " + i + " on savedProperties of " + object);
            }
        }
    }

    function loadSimulation(fileUrl) {
        console.log("Load state called")

        undoList.length = 0

        undoIdx = 1
        undoRecordingEnabled = false

        deleteEverything();

        var code = fileManager.read(fileUrl);
        if(!code) {
            console.log("Load state got empty contents.")
            return;
        }

        var data = JSON.parse(code);

        var createdNodes = [];
        var aliases = [];

        // TODO remove these once simulations no longer contain "connections" and "entities"
        if(data.entities && !data.nodes) {
            console.warn("WARNING: File contains entities, please replace with nodes.")
            data.nodes = data.entities;
        }
        if(data.connections && !data.edges) {
            console.warn("WARNING: File contains connections, please replace with edges.")
            data.edges = data.connections;
        }

        if(!data.nodes) {
            console.warn("ERROR: Could not find nodes. Cannot load simulation " + fileUrl)
            return
        }
        if(!data.edges) {
            console.warn("ERROR: Could not find edges. Cannot load simulation " + fileUrl)
            return
        }

        for(var i in data.nodes) {
            var properties = data.nodes[i];
            if(properties.isAlias && properties.isAlias === true) {
                createdNodes.push({});
                aliases.push({position: i, properties: properties});
                continue;
            }
            var entity = createEntity(properties.fileName, {}, false);
            if(!entity) {
                console.warn("WARNING: Could not create entity of type " + properties.fileName + " while loading " + fileUrl);
                continue;
            }

            applyProperties(entity, properties);
            createdNodes.push(entity);
        }

        for(var i in aliases) {
            var properties = aliases[i].properties;
            var position = aliases[i].position;
            var parent = createdNodes[properties.parent];
            if(!parent) {
                console.warn("ERROR: Could not find parent of alias during file load.");
                continue;
            }
            var entity = parent.resolveAlias(properties.childIndex);
            if(!entity) {
                console.warn("ERROR: Could not resolve alias during file load.")
                continue;
            }
            createdNodes[position] = entity;
        }

        for(var i in data.edges) {
            var edgeProperties = data.edges[i];
            var from = parseInt(edgeProperties.from);
            var to = parseInt(edgeProperties.to);
            if(!createdNodes[from] || !createdNodes[to]) {
                console.warn("WARNING: Cannot connect entities " + from + " and " + to + " while loading " + fileUrl);
                continue;
            }
            connectEntities(createdNodes[from], createdNodes[to]);
        }

        if(data.workspace) {
            applyProperties(workspace, data.workspace);
        }

        undoRecordingEnabled = true
    }

    function addToUndoList() {
        if (!undoRecordingEnabled){
            return
        }
        var fileString = ""

        var counter = 0
//        for(var i in graphEngine.nodes) {
//            var entity = graphEngine.nodes[i]
//            fileString += entity.dump(i)
//        }

//        for(var i in graphEngine.edges) {
//            var connection = graphEngine.edges[i]
//            fileString += connection.dump(i, graphEngine)
//        }

        undoList = undoList.slice(0,undoIdx)
        undoIdx += 1
        undoList.push(fileString)
//        console.log("Making new undolist item ", undoIdx, undoList.length)
        canRedo = false
    }

    function undo(){
        if (undoIdx > 1){
            undoIdx -= 1
            deleteEverything()
//            console.log("Undoing...", undoIdx, undoList.length)
            undoRecordingEnabled = false
            eval(undoList[undoIdx-1])
            undoRecordingEnabled = true
            canRedo = true
        } else {
//            console.log("Nothing to undo! ")
        }
    }

    function redo() {
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
        var nodesToDelete = [];
        for(var i in graphEngine.nodes) {
            nodesToDelete.push(graphEngine.nodes[i])
        }
        for(var i in nodesToDelete) {
            var node = nodesToDelete[i];
            graphEngine.removeNode(node);
        }
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
        var selectedList = selectedEntities
        for(var i in graphEngine.nodes) {
            var listObject = graphEngine.nodes[i]
            listObject.selected = true
            selectedList.push(listObject)
        }
        selectedEntities = selectedList
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

    function deleteNode(node) {
        console.log("Delete node");
        if(selectedEntities.indexOf(node) !== -1) {
            deselectAll();
        }

        if(node.objectName === "retina"){
            camera.retinaCounter -= 1;
        }

        for(var j in node.removableChildren) {
            var child = node.removableChildren[j];
            graphEngine.removeNode(child);
        }

        graphEngine.removeNode(node);
    }

    function deleteEdge(edge) {
        console.log("Delete edge");
        graphEngine.removeEdge(edge);
    }

    function deleteSelected() {
        var toDelete = []
        for(var i in selectedEntities) {
            toDelete.push(selectedEntities[i])
        }
        for(var i in toDelete) {
            var node = toDelete[i]
            console.log("Deleting " + node);
            deleteNode(node);
        }
        if(activeObject && activeObject.objectName === "connection") {
            deleteEdge(activeObject);
        }
        deselectAll()
    }

    function clickedEntity(entity, mouse) {
        if(clickMode === "selection") {
            if(activeObject) {
                activeObject.selected = false
            }
            var selectedList = selectedEntities

            if ((mouse.button === Qt.LeftButton) && (mouse.modifiers & Qt.ShiftModifier)){
                var alreadySelected = false
                for(var j in selectedList) {
                    var alreadySelectedEntity = selectedList[j]
                    if(alreadySelectedEntity ===  entity) {
                        alreadySelected = true
                    }
                }
                if(!alreadySelected) {
                    selectedList.push(entity)
                }
            } else {
                deselectAll()
                selectedList.push(entity)
                entity.selected = true
            }

            for(var i in selectedList) {
                var selectedEntity = selectedList[i]
                selectedEntity.selected = true
            }

            selectedEntities = selectedList

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

    function raiseToTop(node) {
        highestZ += 1.0;
        node.z = highestZ;
    }

    function startedDragEntity(entity) {
        draggedEntity = entity;
    }

    function endedDragEntity(entity) {
        draggedEntity = undefined;
    }

    function createEntity(fileUrl, properties) {
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

        entity.dragStarted.connect(raiseToTop)
        entity.dragStarted.connect(startedDragEntity)
        entity.dragEnded.connect(endedDragEntity)
        entity.clicked.connect(clickedEntity)
        entity.clicked.connect(raiseToTop)
        entity.clickedConnector.connect(clickedConnector)
        entity.droppedConnector.connect(createConnectionToPoint)
        entity.dragProxy = dragProxy
        entity.snapGridSize = Qt.binding(function() {return root.snapGridSize})

        graphEngine.addNode(entity)
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
            console.warn("connectEntities(): Cannot connect an item to itself.")
            return;
        }
        if(connectionExists(itemA, itemB)) {
            console.warn("connectEntities(): Connection already exists.")
            return;
        }
        if(!itemB.canReceiveConnections) {
            console.warn("connectEntities(): " + itemB + " cannot receive connections.")
            return;
        }
        var connection = createConnection(itemA, itemB)
        graphEngine.addEdge(connection)
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
                return Math.min(1.0, Math.max(0.1, scale))
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

            property alias playbackSpeed: playbackControls.playbackSpeed
            property list<PropertyGroup> savedProperties: [
                PropertyGroup {
                    property alias x: workspace.x
                    property alias y: workspace.y
                    property alias scale: workspace.scale
                    property alias playbackSpeed: workspace.playbackSpeed
                }
            ]

            width: 3840
            height: 2160

            scale: 1.1
            transformOrigin: Item.TopLeft


            Item {
                id: dragProxy

                property point previousPosition
                property point gridPosition

                onXChanged: {
                    gridPosition.x = x - x % snapGridSize
                    if(x !== previousPosition.x) {
                        var delta = Qt.point(previousPosition.x - gridPosition.x, 0.0);
                        apply(delta)
                    }
                }
                onYChanged: {
                    gridPosition.y = y - y % snapGridSize
                    if(gridPosition.y !== previousPosition.y) {
                        var delta = Qt.point(0.0, previousPosition.y - gridPosition.y);
                        apply(delta)
                    }
                }

                function moveEntity(entity, delta) {
                    var newX = entity.x - delta.x;
                    var newY = entity.y - delta.y;
                    entity.x = newX - newX % snapGridSize;
                    entity.y = newY - newY % snapGridSize;
                }

                function apply(delta) {

                    if(!draggedEntity) {
                        return
                    }
                    if(selectedEntities.indexOf(draggedEntity) > -1) {
                        for(var i in selectedEntities) {
                            var entity = selectedEntities[i];
                            moveEntity(entity, delta);
                        }
                    } else {
                        moveEntity(draggedEntity, delta);
                    }
                    previousPosition = gridPosition
                }
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

    PlaybackControls {
        id: playbackControls
    }

    CreationMenu {
        id: creationMenu

        blurSource: workspaceFlickable

        onDroppedEntity: {
            var workspacePosition = controlParent.mapToItem(neuronLayer, properties.x, properties.y)
            properties.x = workspacePosition.x
            properties.y = workspacePosition.y
            root.createEntity(fileUrl, properties)
        }

        onDeleteEverything: {
            root.deleteEverything()
        }
    }

    PropertiesPanel {
        id: propertiesPanel
        activeObject: root.activeObject
        workspace: workspace
    }

    ConnectionMenu {
        visible: clickMode === "connection"
        onDoneClicked: {
            clickMode = "selection"
        }
    }

    MainMenu {
        id: mainMenu

        property bool wasRunning: true

        anchors.fill: parent
        blurSource: workspaceFlickable

        onRevealedChanged: {
            if(revealed) {
                wasRunning = root.running
            }
        }

        onContinueClicked: {
            mainMenu.revealed = false
        }

        onNewClicked: {
            root.loadSimulation("qrc:/simulations/empty/empty.nfy")
            mainMenu.revealed = false
        }

        onLoadSimulation: {
            root.loadSimulation(simulation.stateSource)
            mainMenu.revealed = false
        }

        onSaveSimulationRequested: {
            fileManager.showSaveDialog()
        }

        onLoadSimulationRequested: {
            fileManager.showLoadDialog()
        }
    }

    Timer {
        id: timer
        interval: 16
        repeat: true
        running: root.running

        onTriggered: {
            var dt = 0.1e-3 * workspace.playbackSpeed
            time += dt
            graphEngine.step(dt)
        }
    }

    FileManager {
        id: fileManager

        graphEngine: graphEngine
        workspace: workspace

        onLoadState: {
            console.log("Load state signal caught")
            root.loadSimulation(fileUrl)
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

//    Flickable {

//        anchors.fill: parent

//        contentWidth: 2000
//        contentHeight: 2000

//        Column {
//            x: 500
//            y: 300
//            width: 300
//            spacing: 16

//            BoundSlider {
//                text: "Hello"
//                minimumValue: -103.5e-3
//                maximumValue: 51e-3
//                unitScale: 1e-3
//                stepSize: 1e-3
//                unit: "mV"
//                target: texta
//                property: "pos"
//                width: 300
//                precision: 1
//            }

//            BoundSlider {
//                text: "Hello"
//                minimumValue: -103.5e-3
//                maximumValue: 51e-3
//                unitScale: 1e-3
//                stepSize: 1e-3
//                unit: "mV"
//                target: texta
//                property: "pos"
//                width: 300
//                precision: 1
//            }
//        }

//    }

//    Text {
//        id: texta
//        property real pos
//        x: pos * 1000 + 150
//        text: "LOL"
//    }

    Shortcut {
        sequence: "Shift+5"
        onActivated: root.snappingEnabled = !root.snappingEnabled
    }
}
