import QtQuick 2.0
import "paths"

Rectangle {
    id: compartmentRoot

    property bool selected: false
    signal clicked(var compartment)
    signal dragStarted
    property vector2d velocity
    property bool dragging: false

    signal droppedConnectionCreator(var compartment, var connectionCreator)

    property real _baseWidth: 70

    property bool passive: false

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
    property real sodiumActivation: 0.5
    property real sodiumActivationAlpha: 0.1 * ((voltage + 40) / (1 - Math.exp(-((voltage+40)/10))))
    property real sodiumActivationBeta: 4 * Math.exp(-(voltage + 65) / 18.0)
    property real sodiumInactivation: 0.5
    property real sodiumInactivationAlpha: 0.07 * Math.exp(-(voltage + 65) / 20.0)
    property real sodiumInactivationBeta: 1.0 / (Math.exp(-(voltage + 35)/10) + 1.0)
    property real potassiumActivation: 0.5
    property real potassiumActivationAlpha: 0.01 * ((voltage + 55) / (1.0 - Math.exp(-(voltage + 55) / 10.0)))
    property real potassiumActivationBeta: 0.125 * Math.exp(- (voltage + 65) / 80)

    property real axialResistance: 0.5
    property real length: 1.0
    property real diameter: 0.8
//    property real dt: 0.01

    property real sodiumCurrent: 0.0
    property real potassiumCurrent: 0.0
    property real leakCurrent: 0.0

    property var connections: []

    width: _baseWidth * length
    height: _baseWidth * diameter
    radius: Math.min(width, height) / 10
    color: "#6baed6"
    border.color: selected ? "#08306b" : "#2171b5"
    border.width: selected ? 3.0 : 1.0
    antialiasing: true
    smooth: true

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

    function stepForward(dt) {
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

        var gL = meanLeakConductance
        var gNa = meanSodiumConductance
        var gK = meanPotassiumConductance
        var EL = leakPotential
        var ENa = sodiumPotential
        var EK = potassiumPotential
        var V = voltage
        var m3 = m*m*m
        var n4 = n*n*n*n

        if(forceTargetVoltage) {
            voltage = targetVoltage
        } else {

            var axialCurrent = 0
            var d = diameter
            var Ra = axialResistance
            var l = length
            for(var i in connections) {
                var connection = connections[i]
                axialCurrent += d / (4 * Ra * l * l) * (V - connection.otherCompartment(compartmentRoot).voltage)
            }

            leakCurrent = gL * (V - EL)
            if(passive) {
                sodiumCurrent = 0
                potassiumCurrent = 0
            } else {
                sodiumCurrent = gNa * m3 * h * (V - ENa)
                potassiumCurrent = gK * n4 * (V - EK)
            }
            var dV = dt * (1.0 / cm) * (- leakCurrent - sodiumCurrent - potassiumCurrent - axialCurrent)
            _nextVoltage = voltage + dV
        }

//        sodiumCurrent = gNa * m3 * h * (V - ENa)
//        potassiumCurrent = gK * n4 * (V - EK)

        sodiumActivation = m
        sodiumInactivation = h
        potassiumActivation = n
    }

    function finalizeStep() {
        if(!forceTargetVoltage) {
            voltage = _nextVoltage
        }
    }

    onWidthChanged: {
        connectionCreator.resetPosition()
    }
    onHeightChanged: {
        connectionCreator.resetPosition()
    }

    Rectangle {
        anchors.fill: parent
        anchors.margins: 2.0
        radius: parent.radius
        color: "#f7fbff"
        opacity: (voltage + 100) / (150)
    }

    Text {
        anchors.centerIn: parent
        text: voltage.toFixed(1)
        font.pixelSize: 14
    }

    MouseArea {
        anchors.fill: parent
        drag.target: parent
        onPressed: {
            compartmentRoot.dragging = true
            dragStarted()
        }

        onClicked: {
            compartmentRoot.clicked(compartmentRoot)
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
        color: "#4292c6"
        startPoint: Qt.point(compartmentRoot.width / 2, compartmentRoot.height / 2)
        endPoint: Qt.point(connectionCreator.x + connectionCreator.width / 2, connectionCreator.y + connectionCreator.width / 2)
    }

    Item {
        id: connectionCreator

        Component.onCompleted: {
            resetPosition()
        }

        function resetPosition() {
            connectionCreator.x = compartmentRoot.width - width / 2
            connectionCreator.y = compartmentRoot.height - height / 2
        }

        width: compartmentRoot._baseWidth * 0.37
        height: width

        Rectangle {
            anchors.centerIn: parent
            width: parent.width / 2.0
            height: width
            color: "#4292c6"
            border.color: "#f7fbff"
            border.width: 1.0
            radius: width
        }

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
