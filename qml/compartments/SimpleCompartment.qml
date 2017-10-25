import QtQuick 2.0
import Neuronify 1.0

import ".."
import "../edges"

Node {
    id: root
    filename: "compartments/SimpleCompartment.qml"
    name: "Simple compartment"

    readonly property real voltage: engine.voltage

    width: 64
    height: 64
    color: "#dd5900"
    canReceiveConnections: true
    preferredEdge: CompartmentConnection {}

    engine: NodeEngine {
        id: engine

        property bool passive: false

        property real targetVoltage: 0.0
        property bool forceTargetVoltage: false

        property real voltage: 0.0
        property real _nextVoltage: 0.0
        property real meanSodiumConductance: 120
        property real meanPotassiumConductance: 36
        property real meanLeakConductance: 0.3
        property real cm: 1.0
        property real sodiumPotential: 50
        property real potassiumPotential: -77
        property real leakPotential: -54.4
        property real sodiumActivation: 0.5
        property real sodiumInactivation: 0.5
        property real potassiumActivation: 0.5

        property real sodiumCurrent: 0.0
        property real potassiumCurrent: 0.0
        property real leakCurrent: 0.0
        property real receivedCurrents: 0.0
        property bool clamp: false

        savedProperties: PropertyGroup {
//            property alias currentOutput: engine.currentOutput
        }

        onReceivedCurrent: {
            receivedCurrents += current
        }

        onStepped: {
            dt = 0.01

            var V = voltage * 1e3

            var sodiumActivationAlpha = 0.1 * ((V + 40) / (1 - Math.exp(-((V+40)/10))))
            var sodiumActivationBeta = 4 * Math.exp(-(V + 65) / 18.0)
            var sodiumInactivationAlpha = 0.07 * Math.exp(-(V + 65) / 20.0)
            var sodiumInactivationBeta = 1.0 / (Math.exp(-(V + 35)/10) + 1.0)
            var potassiumActivationAlpha = 0.01 * ((V + 55) / (1.0 - Math.exp(-(V + 55) / 10.0)))
            var potassiumActivationBeta = 0.125 * Math.exp(- (V + 65) / 80)

            var m = sodiumActivation
            var alpham = sodiumActivationAlpha
            var betam = sodiumActivationBeta
            var dm = dt * (alpham * (1 - m) - betam * m)
            var h = sodiumInactivation
            var alphah = sodiumInactivationAlpha
            var betah = sodiumInactivationBeta
            var dh = dt * (alphah * (1 - h) - betah * h)
            var n = potassiumActivation
            var alphan = potassiumActivationAlpha
            var betan = potassiumActivationBeta
            var dn = dt * (alphan * (1 - n) - betan * n)

            m += dm
            h += dh
            n += dn

            m = Math.max(0.0, Math.min(1.0, m))
            h = Math.max(0.0, Math.min(1.0, h))
            n = Math.max(0.0, Math.min(1.0, n))

            var gL = meanLeakConductance
            var gNa = meanSodiumConductance
            var gK = meanPotassiumConductance
            var EL = leakPotential
            var ENa = sodiumPotential
            var EK = potassiumPotential
            var m3 = m*m*m
            var n4 = n*n*n*n

            if(forceTargetVoltage) {
                voltage = targetVoltage
            } else {
                leakCurrent = -gL * (V - EL)
                if(passive) {
                    sodiumCurrent = 0
                    potassiumCurrent = 0
                } else {
                    sodiumCurrent = -gNa * m3 * h * (V - ENa)
                    potassiumCurrent = -gK * n4 * (V - EK)
                }
                var dV = dt * (1.0 / cm) * (leakCurrent + sodiumCurrent + potassiumCurrent + receivedCurrents * 1e9)

                V += dV
                voltage = V * 1e-3
            }

    //        sodiumCurrent = gNa * m3 * h * (V - ENa)
    //        potassiumCurrent = gK * n4 * (V - EK)

            sodiumActivation = m
            sodiumInactivation = h
            potassiumActivation = n
            receivedCurrents = 0
        }

        onResettedDynamics: {
//            voltage = 0
//            sodiumActivation = 0
//            potassiumActivation = 0
//            sodiumInactivation = 0
        }
    }

//    controls: Component {
//        PropertiesPage {
//            BoundSlider {
//                target: engine
//                property: "currentOutput"
//                text: "Current output"
//                unit: "nA"
//                minimumValue: 0.0e-9
//                maximumValue: 30.0e-9
//                stepSize: 0.01e-9
//                unitScale: 1e-9
//                precision: 2
//            }
//        }
//    }

    Rectangle {
        anchors.fill: parent
        radius: Math.min(width, height) / 10
        color: "#6baed6"
        border.color: selected ? "#08306b" : "#2171b5"
        border.width: selected ? 3.0 : 1.0
    }

    Rectangle {
        anchors.fill: parent
        anchors.margins: 2.0
        radius: Math.min(width, height) / 10
        color: "#f7fbff"
        opacity: (engine.voltage * 1e3 + 100) / (150)
    }

    Text {
        anchors.centerIn: parent
        text: (engine.voltage * 1e3).toFixed(1)
        font.pixelSize: 14
    }

    DropArea {
        id: dropArea
        anchors {
            fill: parent
            margins: -16
        }
        keys: ["connector"]
        onDropped: {
            receivedDrop(drop.source.node)
        }
    }

    Connector {
        color: "#dd5000"
        connectorColor: "#dd5000"
    }
}
