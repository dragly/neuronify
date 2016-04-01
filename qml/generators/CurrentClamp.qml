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
    property alias currentOutputScaled: engine.currentOutputScaled

    fileName: "generators/CurrentClamp.qml"

    width: 64
    height: 64
    color: "#dd5900"
    canReceiveConnections: false

    engine: NodeEngine {
        id: engine
        property real currentOutputScaled: 10.0
        currentOutput: currentOutputScaled * 1e-6
    }

    controls: Component {
        PropertiesPage {
            title: "Current clamp"
            BoundSlider {
                target: engine
                property: "currentOutputScaled"
                text: "Current output"
                unit: "uA"
                minimumValue: 0.0
                maximumValue: 20.0
            }
        }
    }


    savedProperties: PropertyGroup {
        property alias currentOutputScaled: engine.currentOutputScaled
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

