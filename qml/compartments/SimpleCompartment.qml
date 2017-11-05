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

    engine: CompartmentEngine {
        id: engine

        savedProperties: PropertyGroup {
//            property alias currentOutput: engine.currentOutput
        }

        LeakCurrent {

            voltage: engine.voltage
            restingPotential: -54.4e-3

            onResettedProperties: {
                var conductance = 0.3 * 1e-6
                resistance = (1.0 / conductance)
            }
        }

        SodiumCurrent {
            voltage: engine.voltage
        }

        PotassiumCurrent {
            voltage: engine.voltage
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
