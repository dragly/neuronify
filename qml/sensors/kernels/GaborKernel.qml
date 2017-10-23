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
                property: "shift_x"
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
                property: "shift_y"
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
        property alias shift_x: gaborEngine.shift_x
        property alias shift_y: gaborEngine.shift_y
    }
}

