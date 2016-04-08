import QtQuick 2.0
import Neuronify 1.0

import ".."
import "../controls"
import "../hud"

Edge {
    objectName: "CurrentSynapse"
    filename: "edges/CurrentSynapse.qml"

    engine: EdgeEngine {
        id: engine

        property real current
        property real tau
        property real maximumCurrent
        savedProperties: [
            PropertyGroup {
                property alias current: engine.current
                property alias tau: engine.tau
                property alias maximumCurrent: engine.maximumCurrent
            }
        ]

        onStepped:{
            current -= current/tau * dt;
            currentOutput = current;
        }

        onReceivedFire: {
            current += maximumCurrent;
        }

        onResettedDynamics: {
            current = 0.0
        }

        onResettedProperties: {
            tau = 0.3e-3
            maximumCurrent = 10.0e-9
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
                unit: "nA"
                minimumValue: 0e-9
                maximumValue: 20e-9
                unitScale: 1e-9
                stepSize: 0.1e-9
                precision: 1
            }
            BoundSlider {
                target: engine
                property: "tau"
                text: "Time constant"
                unit: "ms"
                minimumValue: 0.1e-3
                maximumValue: 5e-3
                unitScale: 1e-3
                stepSize: 1e-4
                precision: 2
            }
        }
    }
}
