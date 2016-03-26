import QtQuick 2.0
import "../paths"
import "../hud"
import "../tools"
import ".."
import QtCharts 2.0

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
    property var colors: ["#e41a1c", "#377eb8", "#4daf4a", "#984ea3", "#ff7f00", "#a65628", "#f781bf", "#999999"]
    property int currentSeries: 0
    property string mode: "voltage"
    property string title: "mV"

    property real minimumValue: -100.0e-3
    property real maximumValue: 100.0e-3
    property real timeRange: 100.0e-3

    property real timeSinceLastUpdate: 0
    property real lastUpdateTime: 0

    property var series: [
        series1,
        series2,
        series3,
        series4
    ]

    property real time: 0.0

    controls: Component {
        VoltmeterControls {
            voltmeter: voltmeterRoot
        }
    }
    width: 180
    height: 120
    color: "#deebf7"

    engine: NodeEngine {
        onStepped: {
            time += dt
            for(var i in voltmeterRoot.connectionPlots) {
                var connectionPlot = voltmeterRoot.connectionPlots[i]
                var plot = connectionPlot.plot
                var neuron = connectionPlot.connection.itemA
                if(neuron) {
                    if(mode === "voltage" && neuron.voltage) {
                        plot.addPoint(time, neuron.voltage)
                    }
                }
            }
        }
        onReceivedFire: {
            console.log("Received fire " + sender + stimulation)
            for(var i in voltmeterRoot.connectionPlots) {
                var connectionPlot = voltmeterRoot.connectionPlots[i]
                var plot = connectionPlot.plot
                var neuron = connectionPlot.connection.itemA
                if(neuron.engine && neuron.engine === sender) {
                    console.log("Add point")
                    plot.addPoint(time, 100)
                }
            }
        }
    }

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat(["width", "height"])
    }

    onEdgeAdded: {
        if(currentSeries > series.length - 1) {
            currentSeries += 1
            return
        }
        var plot = series[currentSeries]
        console.log("Got series" + plot + currentSeries)
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

        Plot {
            id: series1
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange
        }

        Plot {
            id: series2
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange
        }

        Plot {
            id: series3
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange
        }

        Plot {
            id: series4
            axisX: axisX
            axisY: axisY
            timeRange: voltmeterRoot.timeRange
        }

        ValueAxis {
            id: axisX
            min: voltmeterRoot.time - timeRange
            max: voltmeterRoot.time
            tickCount: 2
            gridVisible: false
        }

        ValueAxis {
            id: axisY
            min: -10.0e-3
            max: 30.0e-3
            tickCount: 2
            gridVisible: false
        }
    }

    ResizeRectangle {}
}
