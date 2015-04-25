import QtQuick 2.0
import QtQuick.Controls 1.0

import Neuronify 1.0

import ".."


Node {
    property alias currentOutput: engine.currentOutput

    fileName: "generators/CurrentClamp.qml"

    width: 62
    height: 62

    engine: NodeEngine {
        id: engine
        currentOutput: 75.0
    }

    controls: Component {
        Item {
            anchors.fill: parent
            Binding {
                target: engine
                property: "currentOutput"
                value: currentOutputSlider.value
            }

            Column {
                anchors.fill: parent
                Text {
                    text: "Current output: " + currentOutputSlider.value.toFixed(0) + " mA"
                }

                Slider {
                    id: currentOutputSlider
                    value: engine.currentOutput
                    minimumValue: 0.0
                    maximumValue: 200.0
                }
            }
        }
    }

    dumpableProperties: [
        "x",
        "y",
        "currentOutput"
    ]

    Image {
        anchors.fill: parent
        fillMode: Image.PreserveAspectFit
        source: "qrc:/images/creators/generators/current_clamp.png"
    }

    Connector {

    }
}

