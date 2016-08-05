import QtQuick 2.0

ListModel {
    ListElement {
        name: "Leaky inhibitory neuron"
        description: "Inhibitory neuron with only leak currents"
        source: "../neurons/LeakyInhibitoryNeuron.qml"
        imageSource: "qrc:/images/neurons/leaky_inhibitory.png"
    }

//    ListElement {
//        name: "Bursting inhibitory neuron"
//        description: "Neuron that bursts on stimulation"
//        source: "../neurons/BurstInhibitoryNeuron.qml"
//        imageSource: "qrc:/images/neurons/burst_inhibitory.png"
//    }

    ListElement {
        name: "Adaptive inhibitory neuron"
        description: "Inhibitory neuron with adapting spike response"
        source: "../neurons/AdaptationInhibitoryNeuron.qml"
        imageSource: "qrc:/images/neurons/adaptive_inhibitory.png"
    }
}
