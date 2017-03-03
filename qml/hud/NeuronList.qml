import QtQuick 2.0

ListModel {
    ListElement {
        name: "Leaky"
        description: "Excitatory neuron with only leak currents"
        source: "qrc:/qml/neurons/LeakyNeuron.qml"
        imageSource: "qrc:/images/neurons/leaky.png"
    }
//    ListElement {
//        name: "Bursting neuron"
//        description: "Neuron that bursts on stimulation"
//        source: "../neurons/BurstNeuron.qml"
//        imageSource: "qrc:/images/neurons/burst.png"
//    }
    ListElement {
        name: "Adaptive"
        description: "Excitatory neuron with adapting spike response"
        source: "qrc:/qml/neurons/AdaptationNeuron.qml"
        imageSource: "qrc:/images/neurons/adaptive.png"
    }
}
