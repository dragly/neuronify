import QtQuick 2.0
import "paths"

Rectangle {
    id: compartmentRoot

    signal clicked(var compartment, var mouse)
    signal dragStarted
    signal droppedConnectionCreator(var compartment, var connectionCreator)

    property vector2d velocity
    property bool selected: false
    property bool dragging: false

    property real targetVoltage: 0.0
    property bool forceTargetVoltage: false

    property real voltage: 0.0
    property real _nextVoltage: 0.0
    property real meanSodiumConductance: 120
    property real meanPotassiumConductance: 36
    property real meanLeakConductance: 0.3
    property real cm: 1.0
    property real sodiumPotential: 50
    property real potassiumPotential: -77
    property real leakPotential: -54.4
    property real sodiumActivation: 0.0
    property real sodiumActivationAlpha: 0.1 * ((voltage + 40) / (1 - Math.exp(-((voltage+40)/10))))
    property real sodiumActivationBeta: 4 * Math.exp(-(voltage + 65) / 18.0)
    property real sodiumInactivation: 0.0
    property real sodiumInactivationAlpha: 0.07 * Math.exp(-(voltage + 65) / 20.0)
    property real sodiumInactivationBeta: 1.0 / (Math.exp(-(voltage + 35)/10) + 1.0)
    property real potassiumActivation: 0.0
    property real potassiumActivationAlpha: 0.01 * ((voltage + 55) / (1.0 - Math.exp(-(voltage + 55) / 10.0)))
    property real potassiumActivationBeta: 0.125 * Math.exp(- (voltage + 65) / 80)
    property real dt: 0.01

    property var connections: []

    function reset() {
        voltage = 0
        sodiumActivation = 0
        potassiumActivation = 0
        sodiumInactivation = 0
    }

    function removeConnection(connection) {
        var connectionIndex = connections.indexOf(connection)
        if(connectionIndex > -1) {
            connections.splice(connectionIndex, 1)
        }
    }

    function stepForward() {
        var m = sodiumActivation
        var alpham = sodiumActivationAlpha
        var betam = sodiumActivationBeta
        var dm = dt * (alpham * (1 - m) - betam * m)
        var h = sodiumInactivation
        var alphah = sodiumInactivationAlpha
        var betah = sodiumInactivationBeta
        var dh = dt * (alphah * (1 - h) - betah * h)
        var n = potassiumActivation
        var alphan = potassiumActivationAlpha
        var betan = potassiumActivationBeta
        var dn = dt * (alphan * (1 - n) - betan * n)

        m += dm
        h += dh
        n += dn

        m = Math.max(0.0, Math.min(1.0, m))
        h = Math.max(0.0, Math.min(1.0, h))
        n = Math.max(0.0, Math.min(1.0, n))

        if(forceTargetVoltage) {
            voltage = targetVoltage
        } else {
            var gL = meanLeakConductance
            var gNa = meanSodiumConductance
            var gK = meanPotassiumConductance
            var EL = leakPotential
            var ENa = sodiumPotential
            var EK = potassiumPotential
            var V = voltage
            var m3 = m*m*m
            var n4 = n*n*n*n

            var axialCurrent = 0
            for(var i in connections) {
                var connection = connections[i]
                axialCurrent += connection.axialConductance * (V - connection.otherCompartment(compartmentRoot).voltage)
            }

            var dV = dt * ((1.0 / cm) * (- gL * (V - EL) - gNa * m3 * h * (V - ENa) - gK * n4 * (V - EK) - axialCurrent))
            _nextVoltage = voltage + dV
        }

        sodiumActivation = m
        sodiumInactivation = h
        potassiumActivation = n
    }

    function finalizeStep() {
        if(!forceTargetVoltage) {
            voltage = _nextVoltage
        }
    }

    color: Qt.rgba((voltage + 100) / (150), 1.0, 1.0, 1.0)

    width: 60
    height: 50
    radius: Math.min(width, height) / 10
    border.color: selected ? "black" : Qt.rgba((voltage + 100) / (150), 0.5, 0.5, 1.0)
    border.width: Math.max(1.0, Math.min(width / height) / 10.0)
    antialiasing: true
    smooth: true

    Text {
        anchors.centerIn: parent
        text: voltage.toFixed(2)
    }

    MouseArea {
        anchors.fill: parent
        drag.target: parent
        onPressed: {
            compartmentRoot.clicked(compartmentRoot, mouse)
            compartmentRoot.dragging = true
            dragStarted()
            console.log("Drag started!")
        }
        onReleased: {
            compartmentRoot.dragging = false
        }

//        onDragChanged: {
//            console.log("Drag!")
//        }
    }

    SCurve {
        z: -1
        startPoint: Qt.point(compartmentRoot.width / 2, compartmentRoot.height / 2)
        endPoint: Qt.point(connectionCreator.x + connectionCreator.width / 2, connectionCreator.y + connectionCreator.width / 2)
    }

    Rectangle {
        id: connectionCreator

        Component.onCompleted: {
            resetPosition()
        }

        function resetPosition() {
            connectionCreator.x = compartmentRoot.width - width / 2
            connectionCreator.y = compartmentRoot.height - height / 2
        }

        color: "blue"
        border.width: 1.0
        border.color: "lightblue"
        radius: width
        width: compartmentRoot.width * 0.25
        height: width

        MouseArea {
            id: connectionCreatorMouseArea
            anchors.fill: parent
            drag.target: parent
            onReleased: {
                compartmentRoot.droppedConnectionCreator(compartmentRoot, connectionCreator)
                connectionCreator.resetPosition()
            }
        }
    }
}
