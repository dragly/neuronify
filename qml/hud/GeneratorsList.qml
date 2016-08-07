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

    ListElement {
        name: "Visual input"
        description: "Generates spikes based on visual input"
        source: "../sensors/Retina.qml"
        imageSource: "qrc:/images/generators/eye.png"
    }

    ListElement {
        name: "Touch sensor"
        description: "Generates spikes in neurons it is connected to based on touch"
        source: "../sensors/TouchSensor.qml"
        imageSource: "qrc:/images/generators/touch_sensor.png"
    }
}
