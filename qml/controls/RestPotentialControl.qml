import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Controls.Styles 1.1

import Neuronify 1.0

import "qrc:/qml"
import "qrc:/qml/neurons"

Column {
    id: root
    property NeuronEngine engine: null

    width: parent.width

    Text {
        text: "Reset the potential:"
    }

    Button {
        text: "Reset"
        onClicked: {
            engine.reset()
        }
    }

    Text {
        text: "Reset the potential of all neurons:"
    }

    Button {
        text: "Reset all"
        onClicked: {
            for (var i in graphEngine.nodes){
                if (graphEngine.nodes[i].isNeuron) {
                    graphEngine.nodes[i].engine.reset()
                }

            }
        }
    }
}
