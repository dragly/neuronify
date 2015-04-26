import QtQuick 2.0

CreationList {
    id: itemRow

    CreationItem {
        name: "Current clamp"
        description: "Generates constant current clamped to neurons."
        source: "../generators/CurrentClamp.qml"
        imageSource: "qrc:/images/generators/current_clamp.png"
    }

    CreationItem {
        name: "Poisson generator"
        description: "Generates random spikes depending on a given firing rate."
        source: "../generators/PoissonGenerator.qml"
        imageSource: "qrc:/images/generators/poisson_generator.png"
    }
}
