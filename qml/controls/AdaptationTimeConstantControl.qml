import QtQuick 2.0
import Neuronify 1.0

import "qrc:/qml/"
import "qrc:/qml/controls"

BoundSlider {
    property AdaptationCurrent current
    target: current
    minimumValue: 0.0
    maximumValue: 50.0e-3
    property: "timeConstant"
    text: "Time constant"
    unit: "ms"
    unitScale: 1e-3
    stepSize: 1e-4
    precision: 1
}
