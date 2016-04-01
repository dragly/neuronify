import QtQuick 2.0

ListModel {
    ListElement {
        name: "Passive inhibitory neuron"
        description: "Inhibitory neuron with only passive currents."
        source: "../neurons/PassiveInhibitoryNeuron.qml"
        imageSource: "qrc:/images/neurons/passive_inhibitory.png"
    }

    ListElement {
        name: "Bursting inhibitory neuron"
        description: "Neuron that bursts on stimulation."
        source: "../neurons/BurstInhibitoryNeuron.qml"
        imageSource: "qrc:/images/neurons/burst_inhibitory.png"
    }

    ListElement {
        name: "Inhibitory adaptation neuron"
        description: "Inhibitory neuron with passive currents and adaptation on firing."
        source: "../neurons/AdaptationInhibitoryNeuron.qml"
        imageSource: "qrc:/images/neurons/adaptive_inhibitory.png"
    }
}
