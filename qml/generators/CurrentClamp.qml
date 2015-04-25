import QtQuick 2.0
import QtQuick.Controls 1.0

import Neuronify 1.0

import ".."


Node {
    width: 62
    height: 62

    Image {
        anchors.fill: parent
        fillMode: Image.PreserveAspectFit
        source: "qrc:/images/creators/generators/current_clamp.png"
    }

    engine: NodeEngine {
        id: currentEngine
        currentOutput: 500.0
    }

    controls: Component {
        Item {
            anchors.fill: parent
            Binding {
                target: currentEngine
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
                    value: currentEngine.currentOutput
                    minimumValue: 0.0
                    maximumValue: 2000.0
                }
            }
        }
    }

    Connector {

    }
}

