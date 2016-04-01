import QtQuick 2.0
import Neuronify 1.0

import "qrc:/"
import "qrc:/qml/"
import "qrc:/qml/controls"
import "qrc:/qml/neurons"

BoundSlider {
    property NeuronEngine engine
    target: engine
    property: "refractoryPeriod"
    text: "Refractory period"
    unit: "ms"
    minimumValue: 0.0e-3
    maximumValue: 40e-3
    unitScale: 1e-3
    stepSize: 1e-3
    precision: 1
}
