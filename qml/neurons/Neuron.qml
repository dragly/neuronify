import QtQuick 2.0
import ".."
import "../paths"
import "../hud"
import Neuronify 1.0
import QtQuick.Controls 1.3
import QtQuick.Particles 2.0

/*!
\qmltype Neuron

\brief The Neuron type is a base for all neurons.

Any implementation of a neuron should use this class as a base
and define its own NeuronEngine.
*/

Node {
    id: root
    objectName: "neuron"
    fileName: "neurons/Neuron.qml"

//    readonly property alias fireOutput: root.engine.fireOutput
//    readonly property alias initialPotential: root.engine.initialPotential
//    readonly property alias restingPotential: root.engine.restingPotential
//    readonly property alias threshold: root.engine.threshold

    readonly property real voltage: root.engine.voltage
    readonly property bool inhibitory: root.engine.fireOutput < 0.0
    readonly property real firingRate: root.engine.firingRate

    property url imageSource
    property url inhibitoryImageSource
    property bool isNeuron: true

    radius: width / 2
    width: 60
    height: width
    color: inhibitory ? "#e41a1c" : "#6baed6"

    controls: Component {
        NeuronControls {
            neuron: root
            engine: root.engine
            onDeleteClicked: {
                root.destroy(1)
            }
        }
    }

    onFired: {
        fireAnimation.restart()
    }

    engine: NeuronEngine {
        id: engine
        PassiveCurrent {
            id: passiveCurrent
        }
    }

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat(
                    ["engine.fireOutput",
                     "engine.initialPotential",
                     "engine.restingPotential",
                     "engine.threshold",
                     "engine.voltage",
                     "engine.capacitance",
                     "engine.synapticConductance",
                     "engine.synapticTimeConstant",
                     "engine.synapsePotential"])
    }

    Image {
        anchors.fill: parent
        source: inhibitory ? inhibitoryImageSource : imageSource
        fillMode: Image.PreserveAspectFit
        smooth: true
        antialiasing: true
    }

    Rectangle {
        property real value: Math.max(0.0, (voltage + 100e-3) / 150e-3)

        anchors.fill: parent
        anchors.margins: value * 6.0
        radius: width * 0.5
        border.color: "#f7fbff"
        color: "transparent"
        border.width: value * 12.0
        smooth: true
        antialiasing: true
        opacity: value * 0.4
    }

    Rectangle {
        id: fireIndicator
        anchors.fill: parent
        anchors.margins: 2.0
        radius: width * 0.5
        color: "#f7fbff"
        opacity: 0.0
        NumberAnimation {
            id: fireAnimation
            running: false
            target: fireIndicator
            property: "opacity"
            from: 1.0
            to: 0.0
            duration: 800
            easing.type: Easing.OutQuad
        }
    }

    Connector {
        curveColor: inhibitory ? "#e41a1c" : "#6baed6"
        connectorColor: inhibitory ? "#e41a1c" : "#6baed6"
    }
}
