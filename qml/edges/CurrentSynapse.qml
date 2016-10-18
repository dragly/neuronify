import QtQuick 2.0
import QtQuick.Particles 2.0

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
            maximumCurrent = 3.0e-9
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
//            emitter.burst(1);
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
            BoundSlider {
                target: engine
                property: "delay"
                text: "Delay"
                unit: "ms"
                minimumValue: 0.0e-3
                maximumValue: 30.0e-3
                unitScale: 1e-3
                stepSize: 1e-4
                precision: 1
            }
        }
    }

//    Emitter {
//        id: emitter
//        property real duration: {
//            var result = 0.0;
//            if(engine.delay > 0 && root.timeStep > 0) {
//                return Math.max(1, engine.delay / (root.timeStep * Style.playbackSpeed) * 16);
//            } else {
//                result = 1;
//            }
//            return result;
//        }
//        system: particleSystem
//        x: root.startPoint.x
//        y: root.startPoint.y
//        emitRate: 0.0
//        lifeSpan: duration
//        velocity: PointDirection {
//            x: (root.endPoint.x - root.startPoint.x) / emitter.duration * 1000;
//            y: (root.endPoint.y - root.startPoint.y) / emitter.duration * 1000;
//        }
//        size: 24 * Style.workspaceScale
//    }

//    Age {
//        id: ageAffector
//        x: root.endPoint.x - width * 0.5
//        y: root.endPoint.y - width * 0.5
//        system: root.particleSystem
//        width: 8
//        height: 8
//        shape: EllipseShape {}
//        lifeLeft: 0.0
//    }
}
