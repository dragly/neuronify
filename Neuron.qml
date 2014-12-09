import QtQuick 2.0
import "paths"
import Nestify 1.0

Rectangle {
    id: neuronRoot

    property bool selected: false
    signal clicked(var neuron, var mouse)
    signal dragStarted
    property vector2d velocity
    property bool dragging: false
    signal droppedConnector(var neuron, var connector)
    property var copiedFrom

    property real adaptationIncreaseOnFire: 1.0
    property alias voltage: engine.voltage
    property real acceleration: 0.0
    property real speed: 0.0
    property alias cm: engine.cm
    property real timeSinceFire: 0.0
    property var connections: []
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

    width: parent.width * 0.015
    height: width
    radius: width / 2
    color: outputStimulation > 0.0 ? "#6baed6" : "#e41a1c"
    border.color: selected ? "#08306b" : "#2171b5"
    border.width: selected ? 3.0 : 1.0

    function reset() {
        engine.reset()
    }

    function addConnection(connection) {
        connections.push(connection)
    }

    function addPassiveConnection(connection) {
        passiveConnections.push(connection)
    }

    function stepForward(dt) {
        timeSinceFire += dt
        checkFire(dt)
        engine.stepForward(dt)
    }

    function removeConnection(connection) {
        var index = connections.indexOf(connection)
        if(index > -1) {
            connections.splice(index, 1)
        }
        index = passiveConnections.indexOf(connection)
        if(index > -1) {
            passiveConnections.splice(index, 1)
        }
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

    NeuronEngine{
        id: engine
    }

    Rectangle {
        anchors.fill: parent
        anchors.margins: 2.0
        radius: parent.radius
        color: "#f7fbff"
        opacity: (voltage + 100) / (150)
    }

//    Text {
//        anchors.centerIn: parent
//        text: voltage.toFixed(1)
//    }


    MouseArea {
        anchors.fill: parent
        drag.target: parent
        onPressed: {
            neuronRoot.dragging = true
            dragStarted()
        }

        onClicked: {
            neuronRoot.clicked(neuronRoot, mouse)
        }

        onReleased: {
            neuronRoot.dragging = false
        }
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
            connector.x = neuronRoot.width / 2 + 0.707*neuronRoot.radius - connector.width / 2
            connector.y = neuronRoot.height / 2 + 0.707*neuronRoot.radius - connector.height / 2
        }

        width: neuronRoot.width * 0.4
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
