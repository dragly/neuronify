import QtQuick 2.0
import "paths"

Rectangle {
    id: neuronRoot

    property bool selected: false
    signal clicked(var compartment)
    signal dragStarted
    property vector2d velocity
    property bool dragging: false
    signal droppedConnector(var neuron, var connector)

    property real adaptationIncreaseOnFire: 1.0
    property real voltage: -100.0
    property real acceleration: 0.0
    property real speed: 0.0
    property real cm: 1.0
    property real timeSinceFire: 0.0
    property var connections: []
    property var passiveConnections: []
//    property var outputNeurons: []
    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property bool firedLastTime: false
    property real synapticConductance: 0.0
    property real adaptationConductance: 0.0
    property real membraneRestingPotential: -65.0
    property real synapsePotential: 50.0
    property real outputStimulation: 4.0
    property real clampCurrent: 0.0
    property bool clampCurrentEnabled: false

    width: parent.width * 0.015
    height: width
    radius: width / 2
    color: outputStimulation > 0.0 ? "#6baed6" : "#e41a1c"
    border.color: selected ? "#08306b" : "#2171b5"
    border.width: selected ? 3.0 : 1.0

    function addConnection(connection) {
        connections.push(connection)
    }

    function addPassiveConnection(connection) {
        passiveConnections.push(connection)
    }

    function stepForward(dt) {
        timeSinceFire += dt
        checkFire(dt)

        var gs = synapticConductance
        var dgs = -gs / 1.0 * dt
        var gadapt = adaptationConductance
        var dgadapt = -gadapt / 1.0 * dt

        var V = voltage
        var Rm = 1.0
        var Em = membraneRestingPotential
        var Es = synapsePotential
        var Is = gs * (V - Es)
        var Iadapt = gadapt * (V - Em)
        var Iauto = 0.0
        if(clampCurrentEnabled) {
            Iauto = clampCurrent
        }

        var voltageChange = 1.0 / cm * (- (V - Em) / Rm) - Is - Iadapt + Iauto
        var dV = voltageChange * dt
        voltage += dV

        synapticConductance = gs + dgs
//        synapticConductance = Math.max(-0.5, synapticConductance)
        adaptationConductance = gadapt + dgadapt
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
            neuronRoot.clicked(neuronRoot)
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
            connector.x = 1.5*neuronRoot.radius - connectorCircle.width / 2
            connector.y = 1.5*neuronRoot.radius - connectorCircle.height / 2
        }

        width: neuronRoot.width * 0.37
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
