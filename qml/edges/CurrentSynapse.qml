import QtQuick 2.0
import Neuronify 1.0

import ".."
import "../controls"
import "../hud"

Edge {
    objectName: "CurrentSynapse"
    filename: "synapses/CurrentSynapse.qml"

    engine: EdgeEngine {
        id: engine

        property real current: 0.0
        property real tau: 1e-3
        property real maximumCurrent: 60e-6

        savedProperties: [
            PropertyGroup {
                property alias current: engine.current
                property alias tau: engine.tau
                property alias maximumCurrent: engine.maximumCurrent
            }
        ]

        onStepped:{
            current -= current/tau * dt
            currentOutput = current
        }

        onReceivedFire: {
            current += maximumCurrent;
        }
    }

    savedProperties: [
        PropertyGroup {
            property alias engine: engine
        }
    ]

    controls: Component {
        PropertiesPage {
            property string title: "Current based synapse"
            BoundSlider {
                target: engine
                property: "maximumCurrent"
                text: "Maximum current"
                unit: "µA"
                minimumValue: 0e-6
                maximumValue: 100e-6
                unitScale: 1e-6
                stepSize: 1e-7
                precision: 1
            }
            BoundSlider {
                target: engine
                property: "tau"
                text: "Time constant"
                unit: "ms"
                minimumValue: 0.1e-3
                maximumValue: 10e-3
                unitScale: 1e-3
                stepSize: 1e-4
                precision: 2
            }
        }
    }
}
