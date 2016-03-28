import QtQuick 2.0
import "../paths"
import "../hud"
import "../tools"
import ".."
import "../controls"

import QtCharts 2.1
import Neuronify 1.0

/*!
\qmltype RatePlot
\inqmlmodule Neuronify
\ingroup neuronify-meters
\brief Reads the firing rate of the neurons and shows a trace plot

Neurons can connect to the firing rate-meter. When they do, the firing rate-meter shows their rate trace
as a function of time. Each neuron gets spesific color in the firing rate-meter plot.
*/

Node {
    id: ratePlotRoot
    objectName: "ratePlot"
    fileName: "meters/RatePlot.qml"
    square: true
    property var connectionPlots: []
    property var colors: ["#e41a1c", "#377eb8", "#4daf4a", "#984ea3",
        "#ff7f00", "#a65628", "#f781bf", "#999999"]
    property int currentSeries: 0
    property string title: "Hz"

    property real timeFactor: 1000
    property real rateFactor: 1000.

    property real timeRange: 100.0e-3

    property real timeSinceLastUpdate: 0
    property real lastUpdateTime: 0

    property alias maximumValue: axisY.max
    property alias minimumValue: axisY.min

    property alias binLength: engine.binLength

    property real maximumPointCount: {
        if(Qt.platform.os === "android" || Qt.platform.os === "ios") {
            return 80.0
        } else {
            return 240.0
        }
    }

    property real time: 0.0
    property real realTime: 0.0

    property var series: [
        series1,
        series2,
        series3,
        series4
    ]

    controls: Component {
        id: meterContols
        MeterControls {
            meter: ratePlotRoot
            sliderMinimum: 0
            sliderMaximum: 2500
            unit: "Hz"
            meterType: "firing rate"

            BoundSlider {
                id: sliderBinLength
                text: "Bin length"
                minimumValue: 10
                maximumValue: 1000
                target: engine
                property: "binLength"
                stepSize: 10
                unitScale: 1.0


            }
        }

    }
    width: 240
    height: 180
    color: "#deebf7"

    engine: NeuronEngine {
        id: engine
        onStepped: {
            if((realTime - lastUpdateTime) > timeRange / maximumPointCount) {
                time = realTime
                lastUpdateTime = realTime
                for(var i in ratePlotRoot.connectionPlots) {
                    var connectionPlot = ratePlotRoot.connectionPlots[i]
                    var plot = connectionPlot.plot
                    var neuron = connectionPlot.connection.itemA
                    if(neuron) {
//                        if(neuron.firingRate) {
                            plot.addPoint(time * timeFactor,
                                          neuron.firingRate)
//                        }
                    }
                }
            }
            realTime += dt
        }

    }

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat(
                    ["width",
                     "height",
                     "maximumValue",
                     "minimumValue",
                     "binLength"])
    }

    onEdgeAdded: {
        if(currentSeries > series.length - 1) {
            currentSeries += 1
            return
        }
        var plot = series[currentSeries]
        plot.visible = true
        var newList = connectionPlots
        newList.push({connection: edge, plot: plot})
        connectionPlots = newList
        currentSeries += 1
    }

    onEdgeRemoved: {
        for(var i in connectionPlots) {
            var connectionPlot = connectionPlots[i]
            var connectionOther = connectionPlot.connection
            if(connectionOther === edge) {
                connectionPlot.plot.clear()
                connectionPlots.splice(i, 1)
                break
            }
        }
        currentSeries -= 1
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
        id: chartView

        anchors.fill: parent
        legend.visible: false
        antialiasing: true
        backgroundColor: "transparent"
        enabled: false // disables mouse input
        margins.top: 0
        margins.bottom: 0
        margins.left: 0
        margins.right: 0

        Plot {
            id: fireSeries
            axisX: axisX
            axisY: axisY
            timeRange: ratePlotRoot.timeRange * timeFactor
            visible: true
        }

        Plot {
            id: series1
            axisX: axisX
            axisY: axisY
            timeRange: ratePlotRoot.timeRange * timeFactor
        }

        Plot {
            id: series2
            axisX: axisX
            axisY: axisY
            timeRange: ratePlotRoot.timeRange * timeFactor
        }

        Plot {
            id: series3
            axisX: axisX
            axisY: axisY
            timeRange: ratePlotRoot.timeRange * timeFactor
        }

        Plot {
            id: series4
            axisX: axisX
            axisY: axisY
            timeRange: ratePlotRoot.timeRange * timeFactor
        }

        ValueAxis {
            id: axisX
            min: (ratePlotRoot.time - timeRange) * timeFactor
            max: ratePlotRoot.time * timeFactor
            tickCount: 2
            gridVisible: false
            labelFormat: "%.0f"
            labelsFont.pixelSize: 14

        }

        ValueAxis {
            id: axisY
            min: 0.0
            max: 100.
            tickCount: 2
            gridVisible: false
            labelFormat: "%.0f"
            labelsFont.pixelSize: 14
            titleText: ratePlotRoot.title
        }
    }

    ResizeRectangle {}
}
