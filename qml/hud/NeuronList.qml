import QtQuick 2.0

CreationList {
    id: itemRow

    CreationItem {
        name: "Passive neuron"
        description: "Neuron with only passive currents."
        source: "../neurons/PassiveNeuron.qml"
        imageSource: "qrc:/images/neurons/passive.png"
    }

    CreationItem {
        name: "Bursting neuron"
        description: "Neuron that bursts on stimulation."
        source: "../neurons/BurstNeuron.qml"
        imageSource: "qrc:/images/neurons/burst.png"
    }

    CreationItem {
        name: "Adaptation neuron"
        description: "Neuron passive currents and adaptation on firing."
        source: "../neurons/AdaptationNeuron.qml"
        imageSource: "qrc:/images/neurons/adaptive.png"
    }
}
