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
    property alias currentAmplitude: engine.currentOutput

    fileName: "generators/ACClamp.qml"

    width: 62
    height: 62
    color: "#dd5000"

    engine: NodeEngine {
        id: engine
        property real time: 0
        property real amplitude: 75.0
        property real frequency: 1.0
        property real pi: 3.14159
        currentOutput: amplitude*Math.sin(2*pi*frequency*time)
        onStepped: {
            time += dt
        }

    }

    controls: Component {
        Item {
            anchors.fill: parent

            Column {
                anchors.fill: parent
                Text {
                    text: "Current amplitude: " + engine.amplitude.toFixed(0) + " mA"
                }

                BoundSlider {
                    target: engine
                    property: "amplitude"
                    minimumValue: 0.0
                    maximumValue: 200.0
                }
                Text {
                    text: "Frequency: " + 1000*engine.frequency.toFixed(3) + "Hz"
                }

                BoundSlider {
                    target: engine
                    property: "frequency"
                    minimumValue: 0.0
                    maximumValue: 2.0
                }
            }
        }
    }



    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat("currentAmplitude")
    }

    Image {
        anchors.fill: parent
        fillMode: Image.PreserveAspectFit
        source: "qrc:/images/generators/ac_clamp.png"
    }

    Connector {

    }
}
