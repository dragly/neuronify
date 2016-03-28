import QtQuick 2.0
import "../paths"
import "../hud"
import "../tools"
import ".."
import QtCharts 2.1

import Neuronify 1.0

/*!
\qmltype Voltmeter
\inqmlmodule Neuronify
\ingroup neuronify-meters
\brief Reads the voltage of the neurons and shows a trace plot

Neurons can connect to the voltmeter. When they do, the voltmeter shows their voltage trace
as a function of time. Each neuron gets spesific color in the voltmeter plot. To each voltmeter
there is an associated \l{VoltmeterControls} item.
*/

Node {
    id: voltmeterRoot
    objectName: "voltmeter"
    fileName: "meters/Voltmeter.qml"
    square: true
    property var connectionPlots: []
    property var colors: ["#e41a1c", "#377eb8", "#4daf4a", "#984ea3",
        "#ff7f00", "#a65628", "#f781bf", "#999999"]
    property int currentSeries: 0
    property string title: "mV"

    property real timeFactor: 1000
    property real voltageFactor: 1000

    property real timeRange: 100.0e-3

    property real timeSinceLastUpdate: 0
    property real lastUpdateTime: 0

    property alias maximumValue: axisY.max
    property alias minimumValue: axisY.min

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
        MeterControls {
            meter: voltmeterRoot
            sliderMinimum: -250
            sliderMaximum: 250
            unit: "mV"
            meterType: "voltmeter"

        }
    }
    width: 240
    height: 180
    color: "#deebf7"

    engine: NodeEngine {
        onStepped: {
            if((realTime - lastUpdateTime) > timeRange / maximumPointCount) {
                time = realTime
                lastUpdateTime = realTime
                for(var i in voltmeterRoot.connectionPlots) {
                    var connectionPlot = voltmeterRoot.connectionPlots[i]
                    var plot = connectionPlot.plot
                    var neuron = connectionPlot.connection.itemA
                    if(neuron) {
                        if(neuron.voltage) {
                            plot.addPoint(time * timeFactor, neuron.voltage * voltageFactor)
                        }
                    }
                }
            }
            realTime += dt
        }
        onReceivedFire: {
            for(var i in voltmeterRoot.connectionPlots) {
                var connectionPlot = voltmeterRoot.connectionPlots[i]
                var neuron = connectionPlot.connection.itemA
                if(neuron.engine && neuron.engine === sender) {
                    fireSeries.addPoint(time * timeFactor - 1e-1, 1000e-3 * voltageFactor)
                    fireSeries.addPoint(time * timeFactor, neuron.voltage * voltageFactor)
                    fireSeries.addPoint(time * timeFactor + 1e-1, 1000e-3 * voltageFactor)
                }
            }
        }
    }

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat(
                    ["width",
                     "height",
                     "maximumValue",
                     "minimumValue"])
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
            timeRange: voltmeterRoot.timeRange * timeFactor
            visible: true
        }

        Plot {
            id: series1
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange * timeFactor
        }

        Plot {
            id: series2
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange * timeFactor
        }

        Plot {
            id: series3
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange * timeFactor
        }

        Plot {
            id: series4
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange * timeFactor
        }

        ValueAxis {
            id: axisX
            min: (voltmeterRoot.time - timeRange) * timeFactor
            max: voltmeterRoot.time * timeFactor
            tickCount: 2
            gridVisible: false
            labelFormat: "%.0f"
            labelsFont.pixelSize: 14
        }

        ValueAxis {
            id: axisY
            min: -100.0e-3 * voltageFactor
            max: 50.0e-3 * voltageFactor
            tickCount: 2
            gridVisible: false
            labelFormat: "%.0f"
            labelsFont.pixelSize: 14
            titleText: voltmeterRoot.title
        }
    }

    ResizeRectangle {}
}
