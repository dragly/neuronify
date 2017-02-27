import QtQuick 2.0
import Neuronify 1.0

import "../../controls"

RectangularKernelEngine{
    id: rectangularEngine
    resolutionHeight: 20
    resolutionWidth: 20

    property Component controls: Component{
        Column{
            BoundSlider {
                target: rectangularEngine
                property: "orientation"
                minimumValue: 0.0
                maximumValue: 3.*Math.PI/2
                unitScale: Math.PI/180
                stepSize: Math.PI/2.
                precision: 1
                text: "Orientation"
                unit: "degrees"
            }

        }

    }

    savedProperties: PropertyGroup {
        property alias orientation: rectangularEngine.orientation
    }


}



