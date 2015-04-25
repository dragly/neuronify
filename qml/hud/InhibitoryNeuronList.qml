import QtQuick 2.0

CreationList {
    id: itemRow

    CreationItem {
        name: "Passive inhibitory neuron"
        description: "Inhibitory neuron with only passive currents."
        source: "qrc:/qml/neurons/PassiveInhibitoryNeuron.qml"
        imageSource: "qrc:/images/creators/neurons/passive_inhibitory.png"
    }

    CreationItem {
        name: "Bursting inhibitory neuron"
        description: "Neuron that bursts on stimulation."
        source: "qrc:/qml/neurons/BurstNeuron.qml"
        imageSource: "qrc:/images/creators/neurons/burst_inhibitory.png"
    }

    CreationItem {
        name: "Inhibitory adaptation neuron"
        description: "Inhibitory neuron with passive currents and adaptation on firing."
        source: "qrc:/qml/neurons/AdaptationNeuron.qml"
        imageSource: "qrc:/images/creators/neurons/adaptive_inhibitory.png"
    }
}
