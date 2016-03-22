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

    readonly property real fireOutput: root.engine.fireOutput
    readonly property bool inhibitory: root.engine.fireOutput < 0.0
    readonly property real voltage: root.engine.voltage
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
        trailsNormal.burst(10)
    }

    engine: NeuronEngine {
        PassiveCurrent {
            id: passiveCurrent
        }
    }

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat("fireOutput")
    }

    ParticleSystem {
        id: system

        anchors.centerIn: parent

        ImageParticle {
            source: "qrc:/images/neurons/passive.png"
        }

        Emitter {
            id: trailsNormal

            anchors.centerIn: parent
            system: system

            emitRate: 0
            lifeSpan: 800

            velocity: PointDirection {xVariation: 400; yVariation: 400;}
//            acceleration: PointDirection {xVariation: 800; yVariation: 800;}

            velocityFromMovement: 8

            size: 8
            sizeVariation: 4
        }
    }

    Image {
        anchors.fill: parent
        source: inhibitory ? inhibitoryImageSource : imageSource
        fillMode: Image.PreserveAspectFit
        smooth: true
        antialiasing: true
    }

    Rectangle {
        anchors.fill: parent
        anchors.margins: 2.0
        radius: width * 0.5
        color: "#f7fbff"
        opacity: (voltage + 100) / (150)
    }

    Connector {
    }
}
