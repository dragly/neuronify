import QtQuick 2.0
import Neuronify 1.0

import "qrc:/"
import "qrc:/qml/"
import "qrc:/qml/controls"
import "qrc:/qml/neurons"

/*!
\qmltype CapacitanceControl
\inqmlmodule Neuronify
\ingroup neuronify-controls
\brief Control for memebrane capacitance.
*/



BoundSlider {
    property NeuronEngine engine
    target: engine
    property: "capacitance"
    text: "Capacitance"
    unit: "nF"
    minimumValue: 1.0e-9
    maximumValue: 10000e-9
    unitScale: 1e-9
    stepSize: 1e-8
    precision: 1
}
