import QtQuick 2.0

Item {
    id: synapseRoot

    property alias preSynapse: preSynapse
    property alias postSynapse: postSynapse

    property bool selected: false
    signal clicked(var synapse)
    signal dragStarted
    property vector2d velocity
    property bool dragging: false

//    property var connections: []

    width: 100
    height: 70

    function compartmentUnderConnector(position) {
        console.log(position.x)
        if(position.x < 50) {
            return preSynapse
        } else {
            return postSynapse
        }
    }

    function stepForward(dt) {
        preSynapse.stepForward(dt)
        postSynapse.stepForward(dt)
    }

    function finalizeStep() {
        preSynapse.finalizeStep()
        postSynapse.finalizeStep()
    }

    Rectangle {
        id: preSynapse
        property bool selected: parent.selected
        property vector2d velocity
        property var connections: []
        property var passiveConnections: []
        property vector2d connectionPoint: Qt.vector2d(x + synapseRoot.x + width / 2.0, y + synapseRoot.y + width / 2.0)
        property real voltage: 0.0
        property real _nextVoltage: 0.0
        property real leakPotential: -54.4
        property real meanLeakConductance: 0.1

        function addConnection(connection) {
            connections.push(connection)
        }

        function addPassiveConnection(connection) {
            passiveConnections.push(connection)
        }

        function removeConnection(connection) {
            var connectionIndex = connections.indexOf(connection)
            if(connectionIndex > -1) {
                connections.splice(connectionIndex, 1)
                return
            }
            var connectionIndex2 = passiveConnections.indexOf(connection)
            if(connectionIndex2 > -1) {
                passiveConnections.splice(connectionIndex, 1)
                return
            }
        }

        function stepForward(dt) {
            var V = voltage
            var axialCurrent = 0
            var EL = leakPotential
            var gL = meanLeakConductance
            for(var i in connections) {
                var connection = connections[i]
                axialCurrent += 10.0 * (V - connection.otherCompartment(preSynapse).voltage)
            }
            var leakCurrent = gL * (V - EL)
            var dV = dt * (- axialCurrent - leakCurrent)
            _nextVoltage = voltage + dV
        }

        function finalizeStep() {
            voltage = _nextVoltage
        }

        anchors {
            left: parent.left
            verticalCenter: parent.verticalCenter
        }

        color: "#6baed6"
        border.color: selected ? "#08306b" : "#2171b5"
        border.width: selected ? 3.0 : 1.0
        radius: 10
        width: 48
        height: 50

        Text {
            anchors.centerIn: parent
            text: preSynapse.voltage.toFixed(1)
            font.pixelSize: 14
        }
    }

    Rectangle {
        id: postSynapse
        property bool selected: parent.selected
        property vector2d velocity
        property var connections: []
        property var passiveConnections: []
        property vector2d connectionPoint: Qt.vector2d(x + synapseRoot.x + width / 2.0, y + synapseRoot.y + width / 2.0)
        property real voltage: 0.0
        property var spikes: []
        property real spikeTimeConstant: 2.0
        property real _nextVoltage: 0.0
        property bool alreadySpiked: false
        property real synapsePotential: 70.0
        property real timeSinceLastSpikeCleanup: 0.0
        property real leakPotential: -54.4
        property real meanLeakConductance: 3.0
        property real meanSynapseConductance: 10.0

        function addConnection(connection) {
            connections.push(connection)
        }

        function addPassiveConnection(connection) {
            passiveConnections.push(connection)
        }

        function removeConnection(connection) {
            var connectionIndex = connections.indexOf(connection)
            if(connectionIndex > -1) {
                connections.splice(connectionIndex, 1)
                return
            }
            var connectionIndex2 = passiveConnections.indexOf(connection)
            if(connectionIndex2 > -1) {
                passiveConnections.splice(connectionIndex, 1)
                return
            }
        }

        function cleanupSpikes() {
            var oldSpikeExists = true
            while(oldSpikeExists) {
                oldSpikeExists = false
                for(var i in spikes) {
                    var spike = spikes[i]
                    if(spike > 10*spikeTimeConstant) {
                        oldSpikeExists = true
                        spikes.splice(i, 1)
                        break
                    }
                }
            }
        }

        function stepForward(dt) {
            if(preSynapse.voltage > 0.0) {
                if(!alreadySpiked) {
                    spikes.push(0.0)
                }
                alreadySpiked = true
            } else {
                alreadySpiked = false
            }

            var V = voltage
            var EL = leakPotential
            var gL = meanLeakConductance
            var gsMean = meanSynapseConductance
            var axialCurrent = 0
            for(var i in connections) {
                var connection = connections[i]
                axialCurrent += 0.1 * (V - connection.otherCompartment(preSynapse).voltage)
            }
            var gs = 0
            for(var i in spikes) {
                spikes[i] += dt
                var t = spikes[i]
                var tau = spikeTimeConstant
                gs += gsMean * t / tau * Math.exp(- t / tau)
            }
            var Es = synapsePotential
            var synapseCurrent = gs * (V - Es)
            var leakCurrent = gL * (V - EL)

            var dV = dt * (- axialCurrent - synapseCurrent - leakCurrent)
            _nextVoltage = voltage + dV

            if(timeSinceLastSpikeCleanup > 10*spikeTimeConstant) {
                cleanupSpikes()
            }
            timeSinceLastSpikeCleanup += dt
        }

        function finalizeStep() {
            voltage = _nextVoltage
        }

        anchors {
            right: parent.right
            verticalCenter: parent.verticalCenter
        }

        radius: 10
        width: 48
        height: 70
        color: "#6baed6"
        border.color: selected ? "#08306b" : "#2171b5"
        border.width: selected ? 3.0 : 1.0

        Text {
            anchors.centerIn: parent
            text: postSynapse.voltage.toFixed(1)
            font.pixelSize: 14
        }
    }

    MouseArea {
        anchors.fill: parent
        drag.target: parent
        onClicked: {
            synapseRoot.clicked(synapseRoot)
        }
        onPressed: {
            synapseRoot.dragging = true
            dragStarted()
        }

        onReleased: {
            synapseRoot.dragging = false
        }
    }
}
