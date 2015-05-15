import QtQuick 2.0
import ".."
import "../paths"
import "../hud"
import Neuronify 1.0
import QtQuick.Controls 1.3

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

    engine: NeuronEngine {
        PassiveCurrent {
            id: passiveCurrent
        }
    }

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat("fireOutput")
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
