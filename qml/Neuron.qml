import QtQuick 2.0
import "paths"
import "hud"
import Neuronify 1.0

Entity {
    id: neuronRoot

    signal droppedConnector(var neuron, var connector)

    property real adaptationIncreaseOnFire: 1.0
    property alias voltage: engine.voltage
    property real acceleration: 0.0
    property real speed: 0.0
    property alias cm: engine.cm
    property real timeSinceFire: 0.0
    property var passiveConnections: []
    //    property var outputNeurons: []
    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property bool firedLastTime: false
    property alias synapticConductance: engine.synapticConductance
    property alias adaptationConductance: engine.adaptationConductance
    property alias membraneRestingPotential: engine.membraneRestingPotential
    property alias synapsePotential: engine.synapsePotential
    property real outputStimulation: 4.0
    property alias clampCurrent: engine.clampCurrent
    property alias clampCurrentEnabled: engine.clampCurrentEnabled

    controls: Component {
        NeuronControls {
            neuron: neuronRoot
            onDisconnectClicked: {
                simulatorRoot.disconnectNeuron(neuron)
            }
            onDeleteClicked: {
                neuronRoot.destroy()
            }
        }
    }

    objectName: "neuron"
    selected: false
    radius: width / 2
    width: parent.width * 0.015
    height: width
    color: outputStimulation > 0.0 ? "#6baed6" : "#e41a1c"

    Component.onDestruction: {
        _deleteAllConnectionsInList(passiveConnections)
    }

    function reset() {
        engine.reset()
    }

    function addPassiveConnection(connection) {
        passiveConnections.push(connection)
    }

    function stepForward(dt) {
        timeSinceFire += dt
        checkFire(dt)
        engine.stepForward(dt)
    }

    function finalizeStep(dt) {

    }

    function stimulate(stimulation) {
        synapticConductance += stimulation
    }

    function fire() {
        for(var i in connections) {
            var neuron = connections[i].itemB
            neuron.stimulate(outputStimulation)
        }

        adaptationConductance += adaptationIncreaseOnFire

        voltage += 100.0
        timeSinceFire = 0.0
        firedLastTime = true
        //        synapticConductance = 0.0
    }

    function checkFire(dt) {
        if(firedLastTime) {
            voltage = -100
            firedLastTime = false
            return
        }

        var shouldFire = false
        if(voltage > 0.0) {
            shouldFire = true
        }
        if(shouldFire) {
            fire()
        }

    }

    onSimulatorChanged: {
        if(simulator) {
            droppedConnector.connect(simulator.createConnectionToPoint)
        }
    }

    onConnectionRemoved: {
        var index = passiveConnections.indexOf(connection)
        if(index > -1) {
            passiveConnections.splice(index, 1)
        }
    }

    NeuronEngine{
        id: engine
    }

    Rectangle {
        id: background
        anchors.fill: parent
        radius: width / 2
        color: neuronRoot.color
        border.color: selected ? "#08306b" : "#2171b5"
        border.width: selected ? 3.0 : 1.0
    }

    Rectangle {
        anchors.fill: parent
        anchors.margins: 2.0
        radius: background.radius
        color: "#f7fbff"
        opacity: (voltage + 100) / (150)
    }

    SCurve {
        id: connectorCurve
        z: -1
        color: "#4292c6"
        startPoint: Qt.point(neuronRoot.width / 2, neuronRoot.height / 2)
        endPoint: Qt.point(connector.x + connector.width / 2, connector.y + connector.width / 2)
    }

    Item {
        id: connector

        visible: neuronRoot.selected

        Component.onCompleted: {
            resetPosition()
        }

        function resetPosition() {
            connector.x = neuronRoot.width / 2 + 0.707*background.radius - connector.width / 2
            connector.y = neuronRoot.height / 2 + 0.707*background.radius - connector.height / 2
        }

        width: neuronRoot.width * 0.5
        height: width

        Rectangle {
            id: connectorCircle
            anchors.centerIn: parent
            width: parent.width / 2.0
            height: width
            color: "#4292c6"
            border.color: "#f7fbff"
            border.width: 1.0
            radius: width
        }

        MouseArea {
            id: connectorMouseArea
            anchors.fill: parent
            drag.target: parent
            onReleased: {
                neuronRoot.droppedConnector(neuronRoot, connector)
                connector.resetPosition()
            }
        }
    }
}
