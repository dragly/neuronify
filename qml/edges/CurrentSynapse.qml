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

        property real linear
        property real exponential

        property real current
        property real tau
        property real maximumCurrent
        property real time
        property real delay: 0.0

        property bool alphaFunction: false

        property var triggers: []

        savedProperties: [
            PropertyGroup {
                property alias current: engine.current
                property alias tau: engine.tau
                property alias maximumCurrent: engine.maximumCurrent
            }
        ]

        function trigger() {
            if(alphaFunction) {
                linear = 0.0;
                exponential = Math.exp(1.0);
            } else {
                exponential = 1.0;
            }
        }

        onStepped:{
            currentOutput = current;
            if(alphaFunction) {
                linear = linear + dt / tau;
            }
            exponential = exponential - exponential * dt / tau;
            if(alphaFunction) {
                current = maximumCurrent * linear * exponential;
            } else {
                current = maximumCurrent * exponential;
            }
            if(triggers.length > 0) {
                if(triggers[0] < time) {
                    trigger();
                    var newTriggers = triggers;
                    newTriggers.shift();
                    triggers = newTriggers;
                }
            }
            time += dt;
        }

        onReceivedFire: {
            if(delay > 0.0) {
                triggers.push(time + delay);
            } else {
                trigger();
            }
        }

        onResettedDynamics: {
            current = 0.0
        }

        onResettedProperties: {
            tau = 2.0e-3
            maximumCurrent = 3.0e-9
            delay = 5.0e-3
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
                maximumValue: 50e-9
                unitScale: 1e-9
                stepSize: 0.1e-9
                precision: 1
            }
            BoundSlider {
                target: engine
                property: "tau"
                text: "Time constant"
                unit: "ms"
                minimumValue: 0.01e-3
                maximumValue: 1.0e-3
                unitScale: 1e-3
                stepSize: 1e-4
                precision: 2
            }
        }
    }
}
