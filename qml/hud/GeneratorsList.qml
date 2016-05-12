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
        name: "Irregular spike generator"
        description: "Generates randomly distributed spikes with a given firing rate."
        source: "../generators/IrregularSpikeGenerator.qml"
        imageSource: "qrc:/images/generators/irregular_spike_generator.png"
    }

    ListElement {
        name: "Rhythm generator"
        description: "Generates rhytmic spikes with a given firing rate."
        source: "../generators/RhythmGenerator.qml"
        imageSource: "qrc:/images/generators/rhythm_generator.png"
    }
}
