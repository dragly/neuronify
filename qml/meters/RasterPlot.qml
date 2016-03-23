import QtQuick 2.5
import QtCharts 2.0
import Neuronify 1.0

import "../paths"
import "../hud"
import "../tools"
import "../controls"
import ".."

Node {
    id: rasterRoot

    property real time: 0.0
    property real timeRange: 30.0

    property var neurons: []

    objectName: "rasterplot"
    fileName: "meters/RasterPlot.qml"
    square: true

    width: 180
    height: 120
    color: "lightgrey"

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat(
                    ["width",
                     "height",
                     "timeRange"])
    }

    engine: NodeEngine {
        onStepped: {
            time += dt
        }
        onReceivedFire: {
            for(var i in neurons) {
                var neuron = neurons[i]
                if(neuron.engine === sender) {
                    scroller.append(time, i)
                }
            }
        }
    }

    controls: Component {
        Item {
            BoundSlider {
                target: rasterRoot
                property: "timeRange"
                minimumValue: 1.0
                maximumValue: 1000.0
            }
        }
    }

    onEdgeAdded: {
        var neuron = edge.itemA
        var newList = neurons
        neurons.push(neuron)
        neurons = newList
    }

    onEdgeRemoved: {
        console.log(edge)
        console.log(edge.itemA)
        console.log(edge.itemB)
        console.log("....")
        var neuron = edge.itemA
        var newList = neurons
        var index = newList.indexOf(neuron)
        console.log("Removing")
        console.log(neuron)
        console.log(index)
        if(index > -1) {
            newList.splice(index, 1)
            neurons = newList
        }
    }

    Rectangle {
        anchors.fill: parent
        color: parent.color
        border.color: "#9ecae1"
        border.width: 1.0
        smooth: true
        antialiasing: true
    }

    ChartView {
        anchors.fill: parent
        enabled: false // disable mouse input
        legend.visible: false
        ScatterSeries {
            id: scatterSeries
            borderWidth: 0.2
            markerSize: 8.0
            axisX: ValueAxis {
                min: time - timeRange
                max: time
                tickCount: 0
                labelsVisible: false
                gridVisible: false
                visible: false
            }
            axisY: ValueAxis {
                min: 0.0
                max: 10.0
                tickCount: 0
                labelsVisible: false
                gridVisible: false
                visible: false
            }
        }
        ChartScroller {
            id: scroller
            series: scatterSeries
            timeRange: rasterRoot.timeRange
        }
    }

    ResizeRectangle {}
}
