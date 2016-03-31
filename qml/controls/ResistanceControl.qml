import QtQuick 2.0
import Neuronify 1.0

import "qrc:/qml/"
import "qrc:/qml/controls"

BoundSlider {
    property PassiveCurrent current
    target: current
    property: "resistance"
    minimumValue: 1e3
    maximumValue: 100e3
    unitScale: 1e3
    stepSize: 1e3
    precision: 1
    text: "Membrane resistance"
    unit: "kΩ"
}
