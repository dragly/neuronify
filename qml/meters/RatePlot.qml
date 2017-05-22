import QtQuick 2.0
import QtCharts 2.1

import Neuronify 1.0
import ".."
import "../controls"
import "../edges"
import "../hud"
import "../paths"
import "../tools"
import "../style"

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
    filename: "meters/RatePlot.qml"
    square: true
    name: "Firing rate plot"

    property var colors: ["#e41a1c", "#377eb8", "#4daf4a", "#984ea3",
        "#ff7f00", "#a65628", "#f781bf", "#999999"]
    property int currentSeries: 0
    property string title: "s⁻¹"

    property real timeFactor: 1000
    property real rateFactor: 1000.

    property real timeRange: 100.0e-3

    property real timeSinceLastUpdate: 0
    property real lastUpdateTime: 0

    property alias maximumValue: axisY.max
    property alias minimumValue: axisY.min

    property bool showLegend: true

    property alias windowDuration: rateEngine.windowDuration
    property alias temporalResolution: rateEngine.temporalResolution

    property real maximumPointCount: {
        if(Qt.platform.os === "android" || Qt.platform.os === "ios") {
            return 80.0
        } else {
            return 240.0
        }
    }

    property real time: 0.0
    property real realTime: 0.0

    preferredEdge: MeterEdge {}

    canReceiveConnections: false

    controls: Component {
        id: meterContols
        MeterControls {
            meter: ratePlotRoot
            sliderMinimum: 0
            sliderMaximum: 2500
            unit: "s⁻¹"
            meterType: "Firing rate"

            BoundSlider {
                id: sliderWindowDuration
                text: "Window duration"
                minimumValue: 10e-3
                maximumValue: 1000e-3
                target: rateEngine
                property: "windowDuration"
                stepSize: 10e-3
                unitScale: 1e-3
                unit: "ms"
            }

            BoundSlider {
                id: slidertemporalResolution
                text: "Filter kernel width"
                minimumValue: 10e-3
                maximumValue: 1000e-3
                target: rateEngine
                property: "temporalResolution"
                stepSize: 10e-3
                unitScale: 1e-3
                unit: "ms"
            }
        }
    }

    width: 320
    height: 240
    snapToCenter: false

    color: "#deebf7"

    engine: RateEngine {
        id: rateEngine
        onStepped: {
            if((realTime - lastUpdateTime) > timeRange / maximumPointCount) {
                time = realTime
                lastUpdateTime = realTime
//                console.log(time * timeFactor +"    " + rateEngine.firingRate)

                if(rateEngine.neuronCount > 0){
                    series1.addPoint(time * timeFactor, rateEngine.firingRate)
                }
            }

            realTime += dt
        }

    }

    savedProperties: PropertyGroup {
        property alias width: ratePlotRoot.width
        property alias height: ratePlotRoot.height
        property alias maximumValue: ratePlotRoot.maximumValue
        property alias minimumValue: ratePlotRoot.minimumValue
        property alias windowDuration: ratePlotRoot.windowDuration
        property alias temporalResolution: ratePlotRoot.temporalResolution
        property alias showLegend: ratePlotRoot.showLegend
    }

    onEdgeAdded: {
        rateEngine.neuronCount +=1
        var neuron = edge.itemB;
        neuron.fired.connect(engine.addFireEvent);
    }

    onEdgeRemoved: {
        var neuron = edge.itemB;
        neuron.fired.disconnect(engine.addFireEvent);
        rateEngine.neuronCount -=1
    }


    Rectangle {
        anchors.fill: parent
        color: parent.color
        border.color: Style.meter.border.color
        border.width: Style.meter.border.width
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
            id: series1
            axisX: axisX
            axisY: axisY
            timeRange: ratePlotRoot.timeRange * timeFactor
            visible: true
        }


        ValueAxis {
            id: axisX
            min: (ratePlotRoot.time - timeRange) * timeFactor
            max: ratePlotRoot.time * timeFactor
            tickCount: 2
            gridVisible: false
            labelFormat: "%.0f"
            labelsFont.pixelSize: 14
            titleText: ratePlotRoot.showLegend ? "t [ms]" : ""
            titleFont.weight: Font.Normal
            titleFont.pixelSize: 14
            visible: false // IMPORTANT: Due to a bug in Qt Charts/Qt Graphics View,
            // performance is degraded with time when text changes
            // https://bugreports.qt.io/browse/QTBUG-59040
        }

        ValueAxis {
            id: axisY
            min: 0.0
            max: 100.
            tickCount: 2
            gridVisible: false
            labelFormat: "%.0f"
            labelsFont.pixelSize: 14
            titleText: ratePlotRoot.showLegend ? "f [s⁻¹]" : ""
            titleFont.weight: Font.Normal
            titleFont.pixelSize: 14
        }
    }

    ResizeRectangle {}

    Connector {
        color: Style.meter.border.color
        visible: parent.selected || rateEngine.neuronCount < 1
    }
}
