import QtQuick 2.0

ListModel {
    ListElement {
        name: "Passive inhibitory neuron"
        description: "Inhibitory neuron with only passive currents."
        source: "qrc:/qml/neurons/PassiveInhibitoryNeuron.qml"
        imageSource: "qrc:/images/neurons/passive_inhibitory.png"
    }

    ListElement {
        name: "Bursting inhibitory neuron"
        description: "Neuron that bursts on stimulation."
        source: "qrc:/qml/neurons/BurstInhibitoryNeuron.qml"
        imageSource: "qrc:/images/neurons/burst_inhibitory.png"
    }

    ListElement {
        name: "Inhibitory adaptation neuron"
        description: "Inhibitory neuron with passive currents and adaptation on firing."
        source: "qrc:/qml/neurons/AdaptationInhibitoryNeuron.qml"
        imageSource: "qrc:/images/neurons/adaptive_inhibitory.png"
    }
}
