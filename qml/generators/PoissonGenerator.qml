import QtQuick 2.0
import QtQuick.Controls 1.4

import Neuronify 1.0

import ".."
import "../controls"
import "../edges"
import "../hud"
import "../paths"

/*!
    \qmltype PoissonGenerator
    \inqmlmodule Neuronify
    \ingroup neuronify-generators
    \brief An spike generator which can supply input spikes to neurons.

    The Poisson generator can be connected to neurons, and will then supply the neurons with spikes
    generated from a poisson process.
    The generator has a control panel where you can adjust the firing rate and stimulation output, as
    well as whether or not the generated spikes are inhibitory or excitatory.
*/

Node {
    id: root

    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property url imageSource: "qrc:/images/generators/poisson_generator_excitatory.png"
    property url inhibitoryImageSource: "qrc:/images/generators/poisson_generator_inhibitory.png"

    property alias rate: engine.rate

    objectName: "poissonGenerator"
    filename: "generators/PoissonGenerator.qml"

    preferredEdge: CurrentSynapse {}

    width: 64
    height: 64
    color: inhibitory ? "#e41a1c" : "#6baed6"
    canReceiveConnections: false

    engine: NodeEngine {
        id: engine
        property real rate

        savedProperties: PropertyGroup {
            property alias rate: engine.rate
        }

        onStepped: {
            var shouldFire = (Math.random() < rate*dt)
            if(shouldFire) {
                fire()
                overlayAnimation.restart()
            }
        }

        onResettedProperties: {
            rate = 0.5e3
        }
    }

    controls: Component {
        PropertiesPage {
            title: "Poisson generator"
            BoundSlider {
                target: engine
                property: "rate"
                text: "Rate"
                minimumValue: 0.0e3
                maximumValue: 1.0e3
                unitScale: 1.0e3
                unit: "/ms"
            }
            Text{
                text: "Inhibitory: " + (switchRoot.checked ? " Yes" : " No")
            }
            Switch{
                id: switchRoot
                checked: root.inhibitory
            }

            Binding {
                target: root
                property: "inhibitory"
                value: switchRoot.checked
            }


        }
    }

    Image {
        anchors.fill: parent

        source: inhibitory ? inhibitoryImageSource : imageSource
        smooth: true
        antialiasing: true
        fillMode: Image.PreserveAspectFit
    }

    Image {
        id: overlay
        anchors.fill: parent

        source: "qrc:/images/generators/generator_overlay.png"
        smooth: true
        antialiasing: true
        fillMode: Image.PreserveAspectFit
        opacity: 0
    }

    NumberAnimation {
        id: overlayAnimation
        target: overlay
        property: "opacity"
        from: 0.5
        to: 0
        duration: 200
        easing.type: Easing.OutQuad
    }

    Connector {
        visible: root.selected
        color: inhibitory ? "#e41a1c" : "#6baed6"
        connectorColor: inhibitory ? "#e41a1c" : "#6baed6"
    }
}
