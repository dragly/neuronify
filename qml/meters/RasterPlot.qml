import QtQuick 2.5
import QtCharts 2.1
import Neuronify 1.0

import "../paths"
import "../hud"
import "../tools"
import "../controls"
import ".."

Node {
    id: rasterRoot

    property real time: 0.0
    property real timeRange: 10.0e-3
    property real timeScale: 1e-3
    property bool showLegend: true

    property var neurons: []


    objectName: "rasterplot"
    fileName: "meters/RasterPlot.qml"
    square: true

    width: 180
    height: 120
    color: "#deebf7"

    savedProperties: PropertyGroup {
        property alias width: rasterRoot.width
        property alias height: rasterRoot.height
        property alias timeRange: rasterRoot.timeRange
        property alias showLegend: rasterRoot.showLegend
    }

    engine: NodeEngine {
        onStepped: {
            time += dt
        }
        onReceivedFire: {
            for(var i in neurons) {
                var neuron = neurons[i]
                if(neuron.engine === sender) {
                    scroller.append(time / timeScale, parseFloat(i) + 1.0)
                }
            }
        }
    }

    controls: Component {
        Column {
            anchors.fill: parent
        Text {
            text: "Time range: " + timeRange.toFixed(0)
        }
            BoundSlider {
                target: rasterRoot
                property: "timeRange"
                text: "Time range"
                unit: "ms"
                unitScale: 1e-3
                minimumValue: 1.0e-3
                maximumValue: 100.0e-3
                stepSize: 1.0e-3
            }
        }
    }

    function refreshCategories() {
        var toRemove = []
        for(var i in axisY.categoriesLabels) {
            toRemove.push(axisY.categoriesLabels[i])
        }
        for(var i in toRemove) {
            var label = toRemove[i]
            axisY.remove(label)
        }
        for(var i in neurons) {
            var neuron = neurons[i]
            var position = parseFloat(i) + 1.5
            axisY.append(" " + neuron.label, position)
        }
    }

    onEdgeAdded: {
        var neuron = edge.itemA
        var newList = neurons
        neurons.push(neuron)
        neuron.onLabelChanged.connect(refreshCategories)
        neurons = newList

        refreshCategories()
    }

    onEdgeRemoved: {
        var neuron = edge.itemA
        console.log(neuron)
        var newList = neurons
        var index = newList.indexOf(neuron)
        console.log("Index " + index)
        if(index > -1) {
            newList.splice(index, 1)
            neurons = newList
        }
        neuron.onLabelChanged.disconnect(refreshCategories)

        scatterSeries.clear()
        refreshCategories()
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
        backgroundColor: "transparent"

        margins.top: 0
        margins.bottom: 0
        margins.left: 0
        margins.right: 0

        ScatterSeries {
            id: scatterSeries
            borderWidth: 0.2
            markerSize: 8.0
            axisX: ValueAxis {
                id: axisX
                min: (time - timeRange) / timeScale
                max: time / timeScale
                tickCount: 2
                gridVisible: false
                labelsFont.pixelSize: 14
                labelFormat: "%.0f"
                titleText: rasterRoot.showLegend ? "t [ms]" : ""
            }
            axisY: CategoryAxis {
                id: axisY
                min: 0.0
                max: neurons.length + 1.0
                startValue: 0.5
                gridVisible: false
                tickCount: 0
                lineVisible: false
                labelsFont.pixelSize: 14
//                titleText: rasterRoot.showLegend ? "Cell" : ""
            }
        }
        ChartScroller {
            id: scroller
            series: scatterSeries
            timeRange: 100.0
        }
    }

    ResizeRectangle {}
}
