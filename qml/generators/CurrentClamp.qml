import QtQuick 2.0
import QtQuick.Controls 1.0

import Neuronify 1.0

import ".."
import "../controls"


Node {
    property alias currentOutput: engine.currentOutput

    fileName: "generators/CurrentClamp.qml"

    width: 62
    height: 62
    color: "#dd5900"

    engine: NodeEngine {
        id: engine
        currentOutput: 75.0
    }

    controls: Component {
        Item {
            anchors.fill: parent

            Column {
                anchors.fill: parent
                Text {
                    text: "Current output: " + currentOutput.toFixed(0) + " mA"
                }

                BoundSlider {
                    target: engine
                    property: "currentOutput"
                    minimumValue: 0.0
                    maximumValue: 200.0
                }
            }
        }
    }

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat("currentOutput")
    }

    Image {
        anchors.fill: parent
        fillMode: Image.PreserveAspectFit
        source: "qrc:/images/generators/current_clamp.png"
    }

    Connector {

    }
}

