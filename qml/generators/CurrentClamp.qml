import QtQuick 2.0
import QtQuick.Controls 1.0

import Neuronify 1.0

import ".."
import "../controls"

/*!
    \qmltype CurrentClamp
    \inqmlmodule Neuronify
    \ingroup neuronify-generators
    \brief A direct current generator which can suply input to neurons

    The DC generator can be connected to neurons, and will then suply the neurons with current.
    The generator has a control panel where you can adjust the current output.
*/

Node {

    filename: "generators/CurrentClamp.qml"

    width: 64
    height: 64
    color: "#dd5900"
    canReceiveConnections: false

    engine: NodeEngine {
        id: engine
        currentOutput: 300e-12
        savedProperties: PropertyGroup {
            property alias currentOutput: engine.currentOutput
        }
    }

    controls: Component {
        PropertiesPage {
            title: "Current clamp"
            BoundSlider {
                target: engine
                property: "currentOutput"
                text: "Current output"
                unit: "pA"
                minimumValue: 0.0e-12
                maximumValue: 1000e-12
                stepSize: 1e-12
                unitScale: 1e-12
                precision: 1
            }
        }
    }

    Image {
        anchors.fill: parent
        fillMode: Image.PreserveAspectFit
        source: "qrc:/images/generators/current_clamp.png"
    }

    Connector {
        curveColor: "#dd5000"
        connectorColor: "#dd5000"

    }
}

