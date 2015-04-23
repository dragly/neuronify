import QtQuick 2.0
import Neuronify 1.0

Node {
    width: 100
    height: 100
    radius: width * 0.5

    engine: NeuronEngine {
        PassiveCurrent {
        }
        AdaptationCurrent {
        }
    }

    Rectangle {
        color: "green"
        anchors.fill: parent
        radius: parent.radius
    }

    Connector {

    }
}

