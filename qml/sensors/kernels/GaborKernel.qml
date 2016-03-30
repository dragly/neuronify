import QtQuick 2.0
import Neuronify 1.0
import "../../controls"

GaborKernelEngine{
    id: gaborEngine
    resolutionHeight: 80
    resolutionWidth: 80

    property Component controls: Component{
        Column{
            BoundSlider {
                target: gaborEngine
                property: "theta"
                minimumValue: 0.0
                maximumValue: Math.PI
                unitScale: Math.PI
                stepSize: Math.PI/8
                precision: 1
                text: "Orientation"
                unit: "π"
            }

        }

    }

    savedProperties: PropertyGroup {
        property alias theta: gaborEngine.theta
    }
}

