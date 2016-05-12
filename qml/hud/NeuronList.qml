import QtQuick 2.0

ListModel {
    ListElement {
        name: "Leaky neuron"
        description: "Neuron with only leaky currents."
        source: "../neurons/LeakyNeuron.qml"
        imageSource: "qrc:/images/neurons/leaky.png"
    }
    ListElement {
        name: "Bursting neuron"
        description: "Neuron that bursts on stimulation."
        source: "../neurons/BurstNeuron.qml"
        imageSource: "qrc:/images/neurons/burst.png"
    }
    ListElement {
        name: "Adaptation neuron"
        description: "Leaky currents and adaptation on firing."
        source: "../neurons/AdaptationNeuron.qml"
        imageSource: "qrc:/images/neurons/adaptive.png"
    }
}
