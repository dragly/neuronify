import QtQuick 2.0
import ".."
import "../edges"
import "../hud"
import "../paths"

import Neuronify 1.0
import QtQuick.Controls 1.3
import QtQuick.Particles 2.0

/*!
\qmltype Neuron

\brief The Neuron type is a base for all neurons.

Any implementation of a neuron should use this class as a base
and define its own NeuronEngine.
*/

Node {
    id: root
    objectName: "neuron"
    filename: "neurons/Neuron.qml"

    readonly property real voltage: root.engine.voltage

    property url imageSource
    property url inhibitoryImageSource
    property bool isNeuron: true

    radius: width / 2
    width: 64
    height: width
    color: inhibitory ? "#e41a1c" : "#6baed6"

    preferredEdge: CurrentSynapse {}

    onFired: {
        fireAnimation.restart()
    }

    Image {
        anchors.fill: parent
        source: inhibitory ? inhibitoryImageSource : imageSource
        fillMode: Image.PreserveAspectFit
        smooth: true
        antialiasing: true
    }

    Rectangle {
        property real thresholdRatio: Math.max(0.0, 0.5 * (voltage - engine.initialPotential) / (engine.threshold - engine.initialPotential))
        property real hyperpolarizationRatio: 0.0
        property real effectRatio: Math.max(thresholdRatio, hyperpolarizationRatio)

        anchors.fill: parent
        anchors.margins: 6.0
        border.width: effectRatio * 12.0
        border.color: Qt.rgba(1.0, 1.0, 1.0)
        color: "transparent"
        radius: width * 0.5
        smooth: true
        antialiasing: true
        opacity: effectRatio * 0.4
    }

    Rectangle {
        property real hyperpolarizationRatio: Math.max(0.0, (engine.initialPotential - voltage) / (engine.threshold - engine.initialPotential))

        anchors.centerIn: parent
        width: parent.width * Math.min(1.0, hyperpolarizationRatio)
        height: width
        color: Qt.rgba(0.5, 0.5, 0.5)
        radius: width * 0.5
        smooth: true
        antialiasing: true
        opacity: hyperpolarizationRatio * 0.4
    }

    Rectangle {
        id: fireIndicator
        anchors.fill: parent
        anchors.margins: 2.0
        radius: width * 0.5
        color: "#f7fbff"
        opacity: 0.0
        NumberAnimation {
            id: fireAnimation
            running: false
            target: fireIndicator
            property: "opacity"
            from: 1.0
            to: 0.0
            duration: 600
            easing.type: Easing.OutQuad
        }
    }

    Connector {
        color: inhibitory ? "#e41a1c" : "#6baed6"
        connectorColor: inhibitory ? "#e41a1c" : "#6baed6"
    }
}
