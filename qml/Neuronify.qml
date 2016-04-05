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
    property alias playbackSpeed: playbackControls.playbackSpeed

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
        firstLoadTimer.start()
    }

    function firstLoad() {
        resetStyle();
        var latest = StandardPaths.locate(StandardPaths.AppConfigLocation, "latest.nfy");
        if(latest !== "") {
            loadSimulation("file://" + StandardPaths.writableLocation(StandardPaths.AppConfigLocation) + "/latest.nfy");
        } else {
            loadSimulation("qrc:/simulations/tutorial/tutorial_1_intro/tutorial_1_intro.nfy");
        }
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
        firstLoadTimer.stop() // stop in case we loaded before the initial simulations was loaded
        console.log("Load state called")

        pinchArea.scaleSetByDoubleClick = false;

        playbackControls.revealed = true;
        playbackControlsAutoHideTimer.restart();

        undoList.length = 0;

        undoIdx = 1;
        undoRecordingEnabled = false;

        deleteEverything();

        var code = fileManager.read(fileUrl);
        if(!code) {
            console.log("Load state got empty contents.")
            return;
        }

        var data = JSON.parse(code);

        if(data.fileFormatVersion < 2) {
            console.warn("The file " + fileUrl + " has format version " + data.fileFormatVersion + ". " +
                         "We are now at version 2. Some data may be lost when you save it now, because it will be " +
                         "converted to the newest format.")
        }

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
            workspace.load(data.workspace);
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

    function raiseToTop(node) {
        highestZ += 1.0;
        node.z = highestZ;
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
            retinaLoader.retinaCounter += 1
            properties.videoSurface = retinaLoader.item.videoSurface
        }

        properties.simulator = root
        properties.dragProxy = dragProxy

        var entity = component.createObject(neuronLayer, properties)
        if(!entity) {
            console.error("Could not create entity from component " + fileUrl)
            return
        }

        // properties
        entity.snapGridSize = Qt.binding(function() {
            return root.snapGridSize
        })

        // signals
        entity.clicked.connect(clickedEntity)
        entity.clicked.connect(raiseToTop)
        entity.dragStarted.connect(raiseToTop)

        entity.dragStarted.connect(function(entity) {
            draggedEntity = entity;
        });
        entity.dragEnded.connect(function(entity) {
            draggedEntity = undefined;
        });
        entity.clickedConnector.connect(function() {
            clickMode = "connection";
        });
        entity.droppedConnector.connect(function(itemA, connector) {
            var targetEntity = itemUnderConnector(itemA, connector)
            if(targetEntity) {
                connectEntities(itemA, targetEntity)
            }
        })

        // retina specific
        if(isRetina) {
            entity.Component.destruction.connect(function() {
                retinaLoader.retinaCounter -= 1;
            });
        }

        // finalize
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
        connection.clicked.connect(function(connection) {
            deselectAll();
            activeObject = connection;
            connection.selected = true;
        });
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
            console.warn("connectEntities(): " + itemB.fileName + " cannot receive connections.")
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
        id: viewport
        anchors.fill: parent
    }

    Item {
        id: workspaceFlickable

        anchors.fill: parent

        PinchArea {
            id: pinchArea
            anchors.fill: parent

            property bool scaleSetByDoubleClick: false
            property real previousScale: 1.0

            property point workspaceStart
            property var localStartCenter
            property double startScale: 1.0
            property point pinchStart
            property real minimumScale: 0.1
            property real maximumScale: 2.0

            function clampScale(scale) {
                return Math.max(minimumScale, Math.min(maximumScale, scale));
            }

            function calculateScaleAndPosition(localX, localY, targetScale) {
                var localStartCenter = mapToItem(workspace, localX, localY);
                var scale = pinchArea.clampScale(targetScale);
                var oldScale = workspace.scale;
                workspace.scale = scale;
                var x = localX - localStartCenter.x * workspace.scale;
                var y = localY - localStartCenter.y * workspace.scale;
                workspace.scale = oldScale;
                return {x: x, y: y, scale: scale};
            }

            function scaleAndPosition(localX, localY, targetScale) {
                var result = calculateScaleAndPosition(localX, localY, targetScale);
                workspace.scale = result.scale;
                workspace.x = result.x;
                workspace.y = result.y;
            }

            onPinchStarted: {
                pinchStart = Qt.point(pinch.center.x, pinch.center.y);
                startScale = workspace.scale;
            }

            onPinchUpdated: {
                var newScale = pinch.scale * startScale;
                scaleAndPosition(pinchStart.x, pinchStart.y, newScale);
                pinchArea.scaleSetByDoubleClick = false;
            }

            MouseArea {
                id: workspaceMouseArea

                anchors.fill: parent

                propagateComposedEvents: true
                drag.target: workspace

                onWheel: {
                    if(wheel.modifiers & Qt.ControlModifier) {
                        var targetScale = workspace.scale + wheel.angleDelta.y * 0.001;
                        pinchArea.scaleAndPosition(wheel.x, wheel.y, targetScale);
                        pinchArea.scaleSetByDoubleClick = false;
                    } else {
                        workspace.x += wheel.angleDelta.x * 0.4;
                        workspace.y += wheel.angleDelta.y * 0.4;
                    }
                }

                onClicked: {
                    propertiesPanel.revealed = false;
                    deselectAll();
                    selectedEntities = [];
                }

                onDoubleClicked: {
                    var result;
                    var targetScale;
                    var ratio = 2.4;
                    if(pinchArea.scaleSetByDoubleClick) {
                        targetScale = pinchArea.previousScale;
                        pinchArea.scaleSetByDoubleClick = false;
                    } else {
                        pinchArea.previousScale = workspace.scale;
                        if(workspace.scale / pinchArea.maximumScale > 0.75) {
                            // if zoomed very far in already, zoom out instead
                            targetScale = workspace.scale / ratio;
                        } else {
                            targetScale = workspace.scale * ratio;
                        }
                        pinchArea.scaleSetByDoubleClick = true;
                    }
                    result = pinchArea.calculateScaleAndPosition(mouse.x, mouse.y, targetScale);
                    scaleAnimationX.to = result.x;
                    scaleAnimationY.to = result.y;
                    scaleAnimationScale.to = result.scale;
                    scaleAnimation.restart();
                }
            }
        }

        ParallelAnimation {
            id: scaleAnimation
            property real duration: 400
            property int easingType: Easing.OutQuad
            PropertyAnimation {
                id: scaleAnimationX
                target: workspace
                property: "x"
                duration: scaleAnimation.duration
                easing.type: scaleAnimation.easingType
            }
            PropertyAnimation {
                id: scaleAnimationY
                target: workspace
                property: "y"
                duration: scaleAnimation.duration
                easing.type: scaleAnimation.easingType
            }
            PropertyAnimation {
                id: scaleAnimationScale
                target: workspace
                property: "scale"
                duration: scaleAnimation.duration
                easing.type: scaleAnimation.easingType
            }
        }

        Item {
            id: workspace

            function dump() {
                var mappedRectangle = viewport.mapToItem(workspace, 0, 0,
                                                         workspaceFlickable.width, workspaceFlickable.height)
                return {
                    playbackSpeed: root.playbackSpeed,
                    visibleRectangle: {
                        x: mappedRectangle.x,
                        y: mappedRectangle.y,
                        width: mappedRectangle.width,
                        height: mappedRectangle.height
                    }
                }
            }

            function load(properties) {
                if(properties.playbackSpeed) {
                    playbackControls.playbackSpeed = properties.playbackSpeed;
                } else {
                    playbackControls.playbackSpeed = 1.0;
                }

                var visibleRectangle = properties.visibleRectangle;
                if(visibleRectangle) {
                    // reset workspace
                    workspace.x = 0;
                    workspace.y = 0;

                    var widthRatio = visibleRectangle.width / viewport.width;
                    var heightRatio = visibleRectangle.height / viewport.height;
                    var scale = 1.0 / Math.max(widthRatio, heightRatio);

                    workspace.scale = pinchArea.clampScale(scale);

                    var oldCenterX = visibleRectangle.x + visibleRectangle.width / 2.0;
                    var oldCenterY = visibleRectangle.y + visibleRectangle.height / 2.0;

                    var oldCenterInViewport = workspace.mapToItem(viewport, oldCenterX, oldCenterY);

                    // move old center to (0, 0)
                    var newPosition = Qt.point(-oldCenterInViewport.x, -oldCenterInViewport.y);

                    // move old center, now in (0, 0), to center of viewport
                    newPosition.x += viewport.width / 2.0;
                    newPosition.y += viewport.height / 2.0;

                    workspace.x = newPosition.x;
                    workspace.y = newPosition.y;
                }
            }

            width: 3840
            height: 2160

            scale: 1.0
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

    Column {
        anchors {
            right: parent.right
            top: parent.top
        }
        width: Style.touchableSize * 1.5

        //        spacing: Style.spacing

        spacing: 0

        MainMenuButton {
            id: mainMenuButton
            revealed: !mainMenu.revealed
            onClicked: {
                mainMenu.revealed = true
            }
        }

        CreationMenuButton {
            onClicked: {
                creationMenu.revealed = !creationMenu.revealed
            }
        }

        PlaybackButton {
            id: playbackButton
            onClicked: {
                playbackControlsAutoHideTimer.stop()
                playbackControls.revealed = !playbackControls.revealed
            }
        }

        PropertiesButton {
            revealed: activeObject ? true : false
            onClicked: {
                propertiesPanel.revealed = !propertiesPanel.revealed
            }
        }
    }

    DeleteButton {
        anchors {
            right: parent.right
            bottom: parent.bottom
        }
        revealed: activeObject ? true : false
        onClicked: {
            deleteSelected()
        }
    }

    PlaybackControls {
        id: playbackControls
        revealed: true
        MouseArea {
            anchors.fill: parent
            enabled: playbackControlsAutoHideTimer.running
            propagateComposedEvents: true
            onClicked: {
                playbackControlsAutoHideTimer.stop()
                mouse.accepted = false
            }
        }
        Timer {
            id: playbackControlsAutoHideTimer
            running: root.running
            interval: 2000
            onTriggered: {
                playbackControls.revealed = false
            }
        }
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
            root.loadSimulation(simulation)
            mainMenu.revealed = false
            //root.running = wasRunning
        }

        onSaveSimulation: {
            root.saveState(simulation)
            mainMenu.revealed = false
        }

        onSaveSimulationRequested: {
            fileManager.showSaveDialog()
        }

        onLoadSimulationRequested: {
            fileManager.showLoadDialog()
        }

        onRequestScreenshot: {
            workspaceFlickable.grabToImage(callback)
        }
    }

    Timer {
        id: timer

        property real frameTime: 0.0
        property int counter: 0
        property real lastTime: Date.now();
        property bool calculatePerformance: true

        interval: 16
        repeat: true
        running: root.running

        onTriggered: {
            var dt = 0.1e-3
            for(var i = 0; i < root.playbackSpeed; i++) {
                time += dt
                graphEngine.step(dt)
            }

            if(calculatePerformance) {
                var endTime = Date.now();
                frameTime = 0.95 * frameTime + 0.05 * (endTime - lastTime);
                lastTime = endTime;

                if(counter > 10) {
                    console.log("frameTime: " + frameTime);
                    counter = 0;
                }
                counter += 1;
            }
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

    Loader {
        id: retinaLoader

        property int retinaCounter: 0
        property bool active: retinaCounter > 0

        sourceComponent: active ? retinaComponent : undefined
    }

    Component {
        id: retinaComponent
        Item {
            x: -10
            width: 10
            height: 10
            property alias videoSurface: _videoSurface
            VideoSurface{
                id: _videoSurface
                enabled: root.running
                camera: Camera{
                    id: _camera
                    viewfinder.resolution : Qt.size(1280,720)
                    Component.onCompleted: {
                        _camera.stop()
                    }
                }
                onEnabledChanged: {
                    if(!enabled) {
                        _camera.stop()
                    } else {
                        _camera.start()
                    }
                }
            }
            VideoOutput {
                // dummy output needed for camera to work on Android
                anchors.fill: parent
                enabled: Qt.platform.os === "android" && root.running
                visible: Qt.platform.os === "android" && root.running
                source: _videoSurface.camera
            }
        }

    }

    Settings {
        property alias snappingEnabled: root.snappingEnabled
    }

    Timer {
        // this is needed because workspaceFlickable doesn't have width at onCompleted
        id: firstLoadTimer
        interval: 500
        onTriggered: {
            root.firstLoad();
        }
    }

    Shortcut {
        sequence: "Shift+5"
        onActivated: root.snappingEnabled = !root.snappingEnabled
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
}
