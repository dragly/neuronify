import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Dialogs 1.0
import QtQuick.Window 2.2

import Nestify 1.0

import "hud"
import "menus/mainmenu"
import "style"

/*
  TODO:
  Post-inhibitory rebound
  Langvarig depolarisering ved høy frekvens, via kalsiumkanaler. Deaktiveres ved hemming.
  Bistable
  Gaffel for eksitatoriskce
  Gain-synapser med andre symboler, elektriske synapser tegnet som motstand
  Mulighet til å lagre krets
  Mulighet til å lagre time-trace
  Kjøre ting i skyen
  Aksonene er litt tynne, bør være mer synlige
  Linjen rundt cellene bør være tykkere
  Lyd når cellene fyrer av
  Pinch to zoom
  To plott over hverandre
  Koble to kretser sammen etter å ha zoomet ut
  Mulighet til å lagre moduler
*/

Rectangle {
    id: simulatorRoot

    property real lastOrganizeTime: Date.now()
    property real lastStepTime: Date.now()
    property var connections: [] // List used to deselect all connections
    property var organizedItems: []
    property var organizedConnections: []
    property var neurons: []
    property var selectedNeurons: []
    property var copiedNeurons: []
    property var voltmeters: []
    property real currentTimeStep: 0.0
    property real time: 0.0
    //    property var connections: []

    width: 400
    height: 300
    color: "#deebf7"
    antialiasing: true
    smooth: true
    focus: true

    Component.onCompleted: {
        var previousNeuron = undefined
        var previousNeuron2 = undefined
        for(var i = 0; i < 2; i++) {
            var neuron = createNeuron({x: 300 + i * 100, y: 200 + (Math.random()) * 10})
            if(previousNeuron) {
                connectNeurons(previousNeuron, neuron)
            }
            if(i === 0) {
                neuron.clampCurrent = 75.0
                neuron.clampCurrentEnabled = true
            }
            var voltmeter = createVoltmeter({x: 300 + i * 200, y: 400})
            connectVoltmeterToNeuron(neuron, voltmeter)
            previousNeuron = neuron
        }
    }

    function deleteFromList(list, item) {
        var itemIndex = list.indexOf(item)
        if(itemIndex > -1) {
            list.splice(itemIndex, 1)
        }
    }

    function saveState(fileUrl){
        if (!String.format) {
          String.format = function(format) {
            var args = Array.prototype.slice.call(arguments, 1);
            return format.replace(/{(\d+)}/g, function(match, number) {
              return typeof args[number] != 'undefined'
                ? args[number]
                : match
              ;
            });
          };
        }

        var fileString = ""

        console.log("Saving to " + fileUrl)

        var counter = 0
        for(var i in neurons) {
            var neuron = neurons[i]
            console.log(neuron.x)
            var ss = "var neuron{0} = createNeuron({x: {1}, y: {2}, clampCurrent: {3}, clampCurrentEnabled: {4}, adaptationIncreaseOnFire: {5}, outputStimulation: {6}})"
            ss = String.format(ss,i.toString(),neuron.x, neuron.y, neuron.clampCurrent,
              neuron.clampCurrentEnabled, neuron.adaptationIncreaseOnFire, neuron.outputStimulation)
            console.log(ss)
            fileString += ss + "\n"
        }

        for(var i in neurons) {
            var neuron = neurons[i]
            for(var j in neuron.connections){
                var toNeuron = neuron.connections[j].itemB
                var indexOfToNeuron = neurons.indexOf(toNeuron)
                fileString += String.format("connectNeurons(neuron{0}, neuron{1}) \n",i,indexOfToNeuron)
            }
        }

        for(var i in voltmeters){
            var voltmeter = voltmeters[i]
            fileString += String.format("var voltmeter{0} = createVoltmeter({x: {1}, y:{2}}) \n", i, voltmeter.x, voltmeter.y)
            var neuronIndex = neurons.indexOf(voltmeter.connectionPlots[0].connection.itemA)
            fileString += String.format("connectVoltmeterToNeuron(neuron{0}, voltmeter{1}) \n",neuronIndex, i)
        }



        saveFileIO.source = fileUrl
        saveFileIO.write(fileString)
    }

    function loadState(fileUrl){
        creationControls.autoLayout = false
        deleteEverything()
        console.log("Loading file " + fileUrl)
        loadFileIO.source = fileUrl
        var stateFile = loadFileIO.read()
        console.log(stateFile)
        eval(stateFile)
    }


    //////////////////////// end of save/load ////////////////

    //    compartment.destroy(1)
    function deleteEverything() {
        deleteAllVoltmeters()
        deleteAllNeurons()
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






    function deleteNeuron(neuron) {
        deselectAll()
        disconnectNeuron(neuron)
        deleteFromList(neurons, neuron)
        deleteFromList(organizedItems, neuron)
        neuron.destroy(1)
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
        voltmeter.destroy(1)
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
        connection.destroy(1)
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

    function selectAllNeurons() {
        selectedNeurons = []
        connectionControls.connection = null
        voltmeterControls.voltmeter = null
        neuronControls.neuron = null
        selectAllInList(neurons)
        for(var i in neurons) {
            var neuron = neurons[i]
            selectedNeurons.push(neuron)
        }
    }

    function selectNeurons() {
        connectionControls.connection = null
        voltmeterControls.voltmeter = null
        neuronControls.neuron = null
    }

    function copyNeurons() {
        copiedNeurons = []
        var copiedNeuron = []
        for(var i in selectedNeurons) {
            var neuron = selectedNeurons[i]
            copiedNeuron = neuron
            copiedNeurons.push(copiedNeuron)
        }
        selectedNeurons = []
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
        connectionControls.connection = null
        voltmeterControls.voltmeter = null
        neuronControls.neuron = null
        selectAllInList(connections)
        selectAllInList(voltmeters)
        selectAllInList(neurons)
    }

    function deselectAll() {
        connectionControls.connection = null
        voltmeterControls.voltmeter = null
        neuronControls.neuron = null
        deselectAllInList(connections)
        deselectAllInList(voltmeters)
        deselectAllInList(neurons)
    }

    function clickedNeuron(neuron, mouse) {
        deselectAll()
        neuronControls.neuron = neuron
        neuron.selected = true

        if ((mouse.button === Qt.LeftButton) && (mouse.modifiers & Qt.ShiftModifier)){
            selectNeurons()
            var alreadySelected = false
                for(var j in selectedNeurons) {
                    var alreadySelectedNeuron = selectedNeurons[j]
                    if(alreadySelectedNeuron ===  neuron) {
                        alreadySelected = true
                    }
                }
                if(!alreadySelected) {
                    selectedNeurons.push(neuron)
                    console.log(selectedNeurons.length)
                }

        }else{
            selectedNeurons = []
            selectedNeurons.push(neuron)
            neuron.selected = true
        }

        selectAllInList(selectedNeurons)

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

    function createNeuron(properties) {
        var component = Qt.createComponent("Neuron.qml")
        var neuron = component.createObject(neuronLayer, properties)
        neuron.dragStarted.connect(resetOrganize)
        neuron.widthChanged.connect(resetOrganize)
        neuron.heightChanged.connect(resetOrganize)
        neuron.clicked.connect(clickedNeuron)
        neuron.droppedConnector.connect(createConnectionToPoint)
        neurons.push(neuron)
        organizedItems.push(neuron)
        resetOrganize()
        return neuron
    }

    function createVoltmeter(properties) {
        var component = Qt.createComponent("Voltmeter.qml")
        var voltmeter = component.createObject(neuronLayer, properties)
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

    function connectNeurons(itemA, itemB) {
        var connection = createConnection(itemA, itemB)
        itemA.addConnection(connection)
        itemB.addPassiveConnection(connection)
        organizedConnections.push(connection)
        connections.push(connection)
        resetOrganize()
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
        var targetVoltmeter = itemUnderConnector(voltmeters, itemA, connector)
        if(targetVoltmeter) {
            if(!connectionExists(itemA, targetVoltmeter)) {
                connectVoltmeterToNeuron(itemA, targetVoltmeter)
                return
            }
        }

        var targetNeuron = itemUnderConnector(neurons, itemA, connector)
        if(targetNeuron) {
            if(connectionExists(itemA, targetNeuron)) {
                return
            }
            if(itemA === targetNeuron) {
                return
            }

            connectNeurons(itemA, targetNeuron)
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
        var springLength = simulatorRoot.width * 0.06
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
                var k = simulatorRoot.width * 0.007
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
        //        contentWidth: 3840 // * workspace.scale
        //        contentHeight: 2160 // * workspace.scale

        MouseArea {
            id: workspaceMouseArea
            anchors.fill: parent

            drag.target: workspace

            property vector2d last
            property vector2d imagekk

            onWheel: {
                var relativeMouse = mapToItem(workspace, wheel.x, wheel.y)
                workspaceScale.origin.x = relativeMouse.x
                workspaceScale.origin.y = relativeMouse.y
                workspaceScale.xScale = Math.min(2.0, Math.max(0.1, workspaceScale.xScale + wheel.angleDelta.y * 0.001))
                var newPosition = mapFromItem(workspace, relativeMouse.x, relativeMouse.y)
                workspace.x += wheel.x - newPosition.x
                workspace.y += wheel.y - newPosition.y
            }

            onClicked: {
                //                workspaceScale.origin.x = mouse.x
                //                workspaceScale.origin.y = mouse.y
                deselectAll()
                selectedNeurons = []
            }
        }

        Item {
            id: workspace
            property alias color: workspaceRectangle.color

            width: 3840
            height: 2160

            transform: Scale {
                id: workspaceScale
                yScale: xScale
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
        onCreateNeuron: {
            var workspacePosition = creationControls.mapToItem(neuronLayer, position.x, position.y)
            simulatorRoot.createNeuron(workspacePosition)
        }

        onCreateVoltmeter: {
            var workspacePosition = creationControls.mapToItem(neuronLayer, position.x, position.y)
            simulatorRoot.createVoltmeter(workspacePosition)
        }
        onDeleteEverything: {
            simulatorRoot.deleteEverything()
        }
    }

    NeuronControls {
        id: neuronControls
        onDisconnectClicked: {
            simulatorRoot.disconnectNeuron(neuron)
        }
        onDeleteClicked: {
            simulatorRoot.deleteNeuron(neuron)
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

    MainMenu {
        id: mainMenu
        anchors.fill: parent
        blurSource: workspaceFlickable

        onLoadSimulation: {
            loadState(simulation.stateSource)
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
        running: !mainMenu.revealed
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
            lastStepTime = currentTime
        }
    }

    //////////////////////// save/load ////////////////

     FileIO {
         id: loadFileIO
         source: "none"
         onError: console.log(msg)
     }

     FileIO {
         id: saveFileIO
         source: "none"
         onError: console.log(msg)
     }

     FileDialog {
         id: saveFileDialog
         title: "Please eneter a filename"
         visible : false
         selectExisting: false
         nameFilters: ["Nestify files (*.nfy)", "All files (*)"]

         onAccepted: {
             var fileUrlNew = fileUrl
             var extensionSplit = fileUrlNew.toString().split(".")
             var fileExtension = extensionSplit[extensionSplit.length - 1]
             if(fileExtension !== "nfy") {
                 fileUrlNew = Qt.resolvedUrl(fileUrlNew.toString() + ".nfy")
             }
             saveState(fileUrlNew)
         }
     }

     FileDialog {
         id: loadFileDialog
         title: "Please choose a file"
         visible : false
         nameFilters: ["Nestify files (*.nfy)", "All files (*)"]

         onAccepted: {
             loadState(fileUrl)
         }
     }

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
             if(voltmeterControls.voltmeter) {
                 deleteVoltmeter(voltmeterControls.voltmeter)
             } else if(neuronControls.neuron) {
                 deleteNeuron(neuronControls.neuron)
             }
         }
     }
}
