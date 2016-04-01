import QtQuick 2.0
import QtQuick.Controls 1.1
import QtQuick.Layouts 1.1
import QtQuick.Controls.Styles 1.1

import Neuronify 1.0

import "qrc:/qml"
import "qrc:/qml/controls"
import "qrc:/qml/style"


Column {
    id: root
    property StackView stackView: Stack.view
    property var neuron: null
    property NeuronEngine engine: null

    spacing: Style.control.spacing
    width: parent ? parent.width : 100

    Text {
        text: (neuron.voltage * 1e3).toFixed(0) + " mV"
        anchors.right: parent.right
    }

    Button {
        text: "Push me!"
        onClicked: {
            stackView.push(comp)
        }
    }

    Component {
        id: comp

        RestingPotentialControl{
            engine: root.engine
        }
    }

    //    InitialPotentialControl{
    //        engine: root.engine
    //    }

    //    ThresholdControl{
    //        engine: root.engine
    //    }

    //    CapacitanceControl{
    //        engine: root.engine
    //    }

    //    ResistanceControl{
    //    }


    //    SynapticPotentialControl{
    //        engine: root.engine
    //    }

    //    SynapticTimeConstantControl{
    //        engine: root.engine
    //    }

    //    SynapticOutputControl {
    //        engine: root.engine
    //    }
}
