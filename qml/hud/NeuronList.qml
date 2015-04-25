import QtQuick 2.0

CreationList {
    id: itemRow

    CreationItem {
        name: "Passive neuron"
        description: "Neuron with only passive currents."
        source: "qrc:/qml/neurons/PassiveNeuron.qml"
        imageSource: "qrc:/images/creators/neurons/passive.png"
    }

    CreationItem {
        name: "Bursting neuron"
        description: "Neuron that bursts on stimulation."
        source: "qrc:/qml/neurons/BurstNeuron.qml"
        imageSource: "qrc:/images/creators/neurons/burst.png"
    }

    CreationItem {
        name: "Adaptation neuron"
        description: "Neuron passive currents and adaptation on firing."
        source: "qrc:/qml/neurons/AdaptationNeuron.qml"
        imageSource: "qrc:/images/creators/neurons/adaptive.png"
    }
}
