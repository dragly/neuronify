import QtQuick 2.0
import Neuronify 1.0

import "../../controls"

GaborKernelEngine{
    id: gaborEngine
    resolutionHeight: 20
    resolutionWidth: 20

    savedProperties: PropertyGroup {
        property alias theta: gaborEngine.theta
    }
}

