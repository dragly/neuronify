import QtQuick 2.0
import Neuronify 1.0

import ".."
import "../controls"
import "../hud"
import "../style"

Edge {
    id: root

    property real timeStep

    objectName: "CurrentSynapse"
    filename: "edges/CurrentSynapse.qml"

    engine: EdgeEngine {
        id: engine

        property bool alphaFunction
        property real tau
        property real maximumCurrent
        property real delay: 0.0

        property real time
        property real linear
        property real exponential

        property var triggers: []

        savedProperties: [
            PropertyGroup {
                // properties
                property alias tau: engine.tau
                property alias maximumCurrent: engine.maximumCurrent
                property alias delay: engine.delay
                property alias alphaFunction: engine.alphaFunction

                // dynamics
                property alias linear: engine.linear
                property alias exponential: engine.exponential
                property alias triggers: engine.triggers
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

        onResettedDynamics: {
            linear = 0.0;
            exponential = 0.0;
            triggers.length = 0;
        }

        onResettedProperties: {
            tau = 2.0e-3
            maximumCurrent = 2.0e-9
            delay = 5.0e-3
            alphaFunction = false;
        }

        onStepped:{
            root.timeStep = dt;
            if(alphaFunction) {
                currentOutput = maximumCurrent * linear * exponential;
            } else {
                currentOutput = maximumCurrent * exponential;
            }
            if(alphaFunction) {
                linear = linear + dt / tau;
            }
            exponential = exponential - exponential * dt / tau;
            if(triggers.length > 0) {
                if(triggers[0] < time) {
                    trigger();
                    triggers.shift();
                }
            }
            time += dt;
        }

        onReceivedFire: {
            signalAnimation.restart();
            if(delay > 0.0) {
                triggers.push(time + delay);
            } else {
                trigger();
            }
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
                maximumValue: 10e-9
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
                maximumValue: 6.0e-3
                unitScale: 1e-3
                stepSize: 1e-4
                precision: 2
            }
        }
    }

    Rectangle {
        id: signalRectangle

        width: 12
        height: width
        color: root.color
        radius: width * 0.5

        SequentialAnimation {
            id: signalAnimation
            property real duration: {
                if(engine.delay > 0 && root.timeStep > 0) {
                    return Math.max(200, engine.delay / (root.timeStep * Style.playbackSpeed) * 16);
                } else {
                    return 200;
                }
            }
            ParallelAnimation {
                NumberAnimation {
                    target: signalRectangle
                    property: "x"
                    from: root.startPoint.x - signalRectangle.radius
                    to: root.endPoint.x - signalRectangle.radius
                    duration: signalAnimation.duration
                    easing.type: Easing.InOutQuad
                }
                NumberAnimation {
                    target: signalRectangle
                    property: "y"
                    from: root.startPoint.y - signalRectangle.radius
                    to: root.endPoint.y - signalRectangle.radius
                    duration: signalAnimation.duration
                    easing.type: Easing.InOutQuad
                }
            }
        }
    }
}
