import QtQuick 2.0

ListModel {
    ListElement {
        name: "DC clamp"
        description: "Generates constant current clamped to neurons."
        source: "../generators/CurrentClamp.qml"
        imageSource: "qrc:/images/generators/current_clamp.png"
    }

    ListElement {
        name: "AC clamp"
        description: "Generates alternating current clamped to neurons."
        source: "../generators/ACClamp.qml"
        imageSource: "qrc:/images/generators/ac_clamp.png"
    }

    ListElement {
        name: "Poisson generator"
        description: "Generates random spikes depending on a given firing rate."
        source: "../generators/PoissonGenerator.qml"
        imageSource: "qrc:/images/generators/poisson_generator.png"
    }
}
