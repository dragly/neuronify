import QtQuick 2.0
import QtQuick.Controls 1.0

import Neuronify 1.0

import ".."
import "../controls"

/*!
    \qmltype ACClap
    \inqmlmodule Neuronify
    \ingroup neuronify-generators
    \brief An alternating current generator which can suply input to neurons

    The AC generator can be connected to neurons, and will then suply the neurons with current.
    The generator has a control panel where you can adjust the frequency and amplitude
*/

Node {

    fileName: "generators/ACClamp.qml"

    width: 62
    height: 62
    color: "#dd5000"
    canReceiveConnections: false

    engine: NodeEngine {
        id: engine
        property real time: 0
        property real amplitude: 75.0e-6
        property real frequency: 1.0
        property real pi: 3.14159
        savedProperties: PropertyGroup {
            property alias amplitude: engine.amplitude
            property alias frequency: engine.frequency
            property alias time: engine.time
        }

        currentOutput: amplitude*Math.sin(2*pi*frequency*time)
        onStepped: {
            time += dt
        }

    }

    savedProperties: PropertyGroup {
        property alias engine: engine
    }

    controls: Component {
        Item {
            anchors.fill: parent

            Column {
                anchors.fill: parent
                BoundSlider {
                    target: engine
                    property: "amplitude"
                    minimumValue: 0.0
                    maximumValue: 50.0e-6
                    unitScale: 1e-6
                    unit: "uA"
                    text: "Current amplitude"
                    stepSize: 1e-6
                }
                BoundSlider {
                    target: engine
                    property: "frequency"
                    minimumValue: 0.0
                    maximumValue: 200.0
                    unitScale: 1
                    unit: "Hz"
                    text: "Current frequency"
                    stepSize: 1.
                }
            }
        }
    }

    Image {
        anchors.fill: parent
        fillMode: Image.PreserveAspectFit
        source: "qrc:/images/generators/ac_clamp.png"
    }

    Connector {
        curveColor: "#dd5000"
        connectorColor: "#dd5000"

    }
}
