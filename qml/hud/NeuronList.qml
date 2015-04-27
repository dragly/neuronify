import QtQuick 2.0

ListModel {
    ListElement {
        name: "Passive neuron"
        description: "Neuron with only passive currents."
        source: "../neurons/PassiveNeuron.qml"
        imageSource: "qrc:/images/neurons/passive.png"
    }
    ListElement {
        name: "Bursting neuron"
        description: "Neuron that bursts on stimulation."
        source: "../neurons/BurstNeuron.qml"
        imageSource: "qrc:/images/neurons/burst.png"
    }
    ListElement {
        name: "Adaptation neuron"
        description: "Passive currents and adaptation on firing."
        source: "../neurons/AdaptationNeuron.qml"
        imageSource: "qrc:/images/neurons/adaptive.png"
    }
}
