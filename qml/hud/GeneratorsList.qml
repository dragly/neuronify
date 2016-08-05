import QtQuick 2.0

ListModel {
    ListElement {
        name: "DC current source"
        description: "Generates constant current clamped to neurons"
        source: "../generators/CurrentClamp.qml"
        imageSource: "qrc:/images/generators/current_clamp.png"
    }

    ListElement {
        name: "AC current source"
        description: "Generates alternating current clamped to neurons"
        source: "../generators/ACClamp.qml"
        imageSource: "qrc:/images/generators/ac_clamp.png"
    }

    ListElement {
        name: "Irregular spike generator"
        description: "Generates randomly distributed spikes with a given average firing rate"
        source: "../generators/IrregularSpikeGenerator.qml"
        imageSource: "qrc:/images/generators/irregular_spike_generator.png"
    }

    ListElement {
        name: "Regular spike generator"
        description: "Generates spikes with a constant given firing rate"
        source: "../generators/RegularSpikeGenerator.qml"
        imageSource: "qrc:/images/generators/regular_spike_generator.png"
    }
}
