import QtQuick 2.0

import Neuronify 1.0

import ".."


Node {
    width: 62
    height: 62

    Rectangle {
        anchors.fill: parent
        color: "orange"
    }

    engine: NodeEngine {
        currentOutput: 500.0
    }

    Connector {

    }
}

