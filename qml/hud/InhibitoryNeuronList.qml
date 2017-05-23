import QtQuick 2.0

ListModel {
    ListElement {
        name: "Leaky"
        description: "Inhibitory neuron with only leak currents"
        source: "qrc:/qml/neurons/LeakyInhibitoryNeuron.qml"
        imageSource: "qrc:/images/neurons/leaky_inhibitory.png"
    }

//    ListElement {
//        name: "Bursting inhibitory neuron"
//        description: "Neuron that bursts on stimulation"
//        source: "qrc:/qml/neurons/BurstInhibitoryNeuron.qml"
//        imageSource: "qrc:/images/neurons/burst_inhibitory.png"
//    }

    ListElement {
        name: "Adaptive"
        description: "Inhibitory neuron with adapting spike response"
        source: "qrc:/qml/neurons/AdaptationInhibitoryNeuron.qml"
        imageSource: "qrc:/images/neurons/adaptive_inhibitory.png"
    }
}
