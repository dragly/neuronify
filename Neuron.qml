import QtQuick 2.0

Rectangle {
    id: neuronRoot

    property bool selected: false
    signal clicked(var compartment)
    signal dragStarted
    property vector2d velocity
    property bool dragging: false

    property real voltage: -100.0
    property real acceleration: 0.0
    property real speed: 0.0
    property real cm: 1.0
    property real timeSinceFire: 0.0
    property bool auto: false
    property var outputNeurons: []
    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property bool firedLastTime: false
    property real synapticConductance: 0.0
    property real adaptationConduction: 0.0

    width: parent.width * 0.03
    height: width
    radius: width
    color: "#6baed6"
    border.color: selected ? "#08306b" : "#2171b5"
    border.width: selected ? 3.0 : 1.0

    function addOutput(neuron) {
        outputNeurons.push(neuron)
    }

    function stepForward(dt) {
        checkFire(dt)

        var gs = synapticConductance
        var dgs = -gs / 10.0 * dt

        var V = voltage
        var Rm = 1.0
        var Em = -65.0
        var Is = gs * (V - 50)
        var Ia = 0.0
        if(auto) {
            Ia = -100.0
        }

        var voltageChange = 1.0 / cm * (- (V - Em) / Rm) - Is - Ia
        var dV = voltageChange * dt
        voltage += dV

        for(var i in stimulations) {
            stimulations[i] += dt
        }

        synapticConductance = gs + dgs
        timeSinceFire += dt
    }

    function finalizeStep(dt) {

    }

    function stimulate() {
        synapticConductance += 3.0
    }

    function fire() {
        for(var i in outputNeurons) {
            var neuron = outputNeurons[i]
            neuron.stimulate()
        }


        stimulations.length = 0

        voltage += 100.0
        timeSinceFire = 0.0
        firedLastTime = true
        synapticConductance = 0.0
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

    Text {
        anchors.centerIn: parent
        text: voltage.toFixed(1)
    }


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
}
