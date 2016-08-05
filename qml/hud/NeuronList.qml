import QtQuick 2.0

ListModel {
    ListElement {
        name: "Leaky excitatory neuron"
        description: "Excitatory neuron with with only leak currents"
        source: "../neurons/LeakyNeuron.qml"
        imageSource: "qrc:/images/neurons/leaky.png"
    }
//    ListElement {
//        name: "Bursting neuron"
//        description: "Neuron that bursts on stimulation"
//        source: "../neurons/BurstNeuron.qml"
//        imageSource: "qrc:/images/neurons/burst.png"
//    }
    ListElement {
        name: "Adaptive excitatory neuron"
        description: "Excitatory neuron with adapting spike response"
        source: "../neurons/AdaptationNeuron.qml"
        imageSource: "qrc:/images/neurons/adaptive.png"
    }
}
