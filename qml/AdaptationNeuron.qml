import QtQuick 2.0
import Neuronify 1.0

Neuron {
    width: 100
    height: 100
    radius: width * 0.5

    engine: NeuronEngine {
        id: engine
        PassiveCurrent {
        }
        AdaptationCurrent {
        }
    }

    Connector {
    }

    Rectangle {
        color: "green"
        anchors.fill: parent
        radius: parent.radius
    }
}

