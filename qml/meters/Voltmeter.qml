import QtQuick 2.0
import "qrc:/qml/"
import "qrc:/qml/hud"
import "qrc:/qml/paths"
import "qrc:/qml/tools"
import "qrc:/qml/style"
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
    property bool showLegend: false

    property real timeFactor: 1000
    property real voltageFactor: 1000

    property real timeRange: 100.0e-3

    property real timeSinceLastUpdate: 0
    property real lastUpdateTime: 0

    property alias maximumValue: axisY.max
    property alias minimumValue: axisY.min

    property real maximumPointCount: {
        if(Qt.platform.os === "android" || Qt.platform.os === "ios") {
            return 120.0
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

    property var fireSeries: [
        fireSeries1,
        fireSeries2,
        fireSeries3,
        fireSeries4
    ]

    controls: Component {
        MeterControls {
            meter: voltmeterRoot
            sliderMinimum: -250
            sliderMaximum: 250
            unit: "mV"
            meterType: "Voltmeter"

        }
    }
    width: 320
    height: 224
    color: Style.color.foreground

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
                var firePlot = connectionPlot.firePlot
                var neuron = connectionPlot.connection.itemA
                if(neuron.engine && neuron.engine === sender) {
                    firePlot.addPoint(time * timeFactor - 1e-1, 1000e-3 * voltageFactor)
                    firePlot.addPoint(time * timeFactor, neuron.voltage * voltageFactor)
                    firePlot.addPoint(time * timeFactor + 1e-1, 1000e-3 * voltageFactor)
                }
            }
        }
    }

    savedProperties: PropertyGroup {
        property alias width: voltmeterRoot.width
        property alias height: voltmeterRoot.height
        property alias maximumValue: voltmeterRoot.maximumValue
        property alias minimumValue: voltmeterRoot.minimumValue
        property alias showLegend: voltmeterRoot.showLegend
    }

    onEdgeAdded: {
        if(currentSeries > series.length - 1) {
            currentSeries += 1
            return
        }
        var plot = series[currentSeries]
        plot.visible = true

        var firePlot = fireSeries[currentSeries]
        plot.visible = true

        var newList = connectionPlots
        newList.push({connection: edge, plot: plot, firePlot: firePlot})
        connectionPlots = newList
        currentSeries += 1
    }

    onEdgeRemoved: {
        for(var i in connectionPlots) {
            var connectionPlot = connectionPlots[i]
            var connectionOther = connectionPlot.connection
            if(connectionOther === edge) {
                connectionPlot.plot.clear()
                connectionPlot.firePlot.clear()
                connectionPlots.splice(i, 1)
                break
            }
        }
        currentSeries -= 1
    }

    Rectangle {
        anchors.fill: parent
        color: parent.color
        border.color: Style.border.color
        border.width: Style.border.width
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
            id: fireSeries1
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange * timeFactor
            visible: true
            color: series1.color
        }

        Plot {
            id: fireSeries2
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange * timeFactor
            visible: true
            color: series2.color
        }

        Plot {
            id: fireSeries3
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange * timeFactor
            visible: true
            color: series3.color
        }

        Plot {
            id: fireSeries4
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange * timeFactor
            visible: true
            color: series4.color
        }


        Plot {
            id: series1
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange * timeFactor
            color: colors[0]
        }

        Plot {
            id: series2
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange * timeFactor
            color: colors[1]
        }

        Plot {
            id: series3
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange * timeFactor
            color: colors[2]
        }

        Plot {
            id: series4
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange * timeFactor
            color: colors[3]
        }

        ValueAxis {
            id: axisX
            min: (voltmeterRoot.time - timeRange) * timeFactor
            max: voltmeterRoot.time * timeFactor
            tickCount: 2
            gridVisible: false
            labelFormat: "%.0f"
            labelsFont.pixelSize: 14
            titleText: voltmeterRoot.showLegend ? "t [ms]" : ""
        }

        ValueAxis {
            id: axisY
            min: -100.0e-3 * voltageFactor
            max: 50.0e-3 * voltageFactor
            tickCount: 2
            gridVisible: false
            labelFormat: "%.0f"
            labelsFont.pixelSize: 14
            titleText: voltmeterRoot.showLegend ? "V [mV]" : ""
        }
    }

    ResizeRectangle {}
}
