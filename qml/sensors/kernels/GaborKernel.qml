import QtQuick 2.0
import Neuronify 1.0
import "../../controls"

GaborKernelEngine{
    id: gaborEngine
    resolutionHeight: 20
    resolutionWidth: 20

    property Component controls: Component{
        Column{
            BoundSlider {
                target: gaborEngine
                property: "xOffset"
                minimumValue: -0.5
                maximumValue: 0.5
                unitScale: 0.1
                stepSize: 0.1
                precision: 1
                text: "x"
                unit: ""
            }

            BoundSlider {
                target: gaborEngine
                property: "yOffset"
                minimumValue: -0.5
                maximumValue: 0.5
                unitScale: 0.1
                stepSize: 0.1
                precision: 1
                text: "y"
                unit: ""
            }

            BoundSlider {
                target: gaborEngine
                property: "theta"
                minimumValue: 0.0
                maximumValue: Math.PI
                unitScale: Math.PI/180
                stepSize: Math.PI/8
                precision: 1
                text: "Orientation"
                unit: "degrees"
            }


        }

    }

    savedProperties: PropertyGroup {
        property alias theta: gaborEngine.theta
        property alias xOffset: gaborEngine.xOffset
        property alias yOffset: gaborEngine.yOffset
    }
}

