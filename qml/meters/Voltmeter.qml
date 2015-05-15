import QtQuick 2.0
import "../paths"
import "../hud"
import ".."

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
    property var connectionPlots: []
    property var colors: ["#e41a1c", "#377eb8", "#4daf4a", "#984ea3", "#ff7f00", "#a65628", "#f781bf", "#999999"]
    property int currentColor: 0
    property string mode: "voltage"
    property string title: "mV"

    property real minimumValue: -100.0
    property real maximumValue: 100.0

    property real timeSinceLastUpdate: 0
    property real lastUpdateTime: 0

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
            timeSinceLastUpdate += dt
            var currentUpdateTime = Date.now()
            var timeDiff = (currentUpdateTime - lastUpdateTime) / 1000
            if(timeDiff < 0.010) {
                return
            }

            for(var i in voltmeterRoot.connectionPlots) {
                var connectionPlot = voltmeterRoot.connectionPlots[i]
                var plot = connectionPlot.plot
                var neuron = connectionPlot.connection.itemA
                if(neuron) {
                    if(mode === "voltage" && neuron.voltage) {
                        plot.addPoint(neuron.voltage)
                    }
                }
            }
            lastUpdateTime = currentUpdateTime
            timeSinceLastUpdate = 0
        }
    }

    function resetMinMax(plot) {
        plot.minimumValue = minimumValue
        plot.maximumValue = maximumValue
    }

    function resetAllMinMax() {
        for(var i in connectionPlots) {
            var connectionPlot = connectionPlots[i]
            resetMinMax(connectionPlot.plot)
        }
    }

    onMinimumValueChanged: {
        resetAllMinMax()
    }

    onMaximumValueChanged: {
        resetAllMinMax()
    }

    onEdgeAdded: {
        if(currentColor > colors.length - 1) {
            currentColor = 0
        }
        var color = colors[currentColor]

        var newList = voltmeterRoot.connectionPlots
        var plotComponent = Qt.createComponent("../Plot.qml")
        if(plotComponent.status !== Component.Ready) {
            console.log("Could not create plot component.")
            console.log(plotComponent.errorString())
        }

        var plot = plotComponent.createObject(plotLayer, {strokeStyle: color})
//        connection.color = color
        resetMinMax(plot)
        newList.push({connection: edge, plot: plot})
        voltmeterRoot.connectionPlots = newList
        currentColor += 1
    }

    onEdgeRemoved: {
        for(var i in connectionPlots) {
            var connectionPlot = connectionPlots[i]
            var connectionOther = connectionPlot.connection
            if(connectionOther === edge) {
                connectionPlots.splice(i, 1)
                connectionPlot.plot.destroy(1)
                break
            }
        }
    }

    onModeChanged: {
        switch(mode) {
        case "voltage":
            minimumValue = -100
            maximumValue = 100
            title = "V"
            break
        }
        for(var i in connectionPlots) {
            var connectionPlot = connectionPlots[i]
            resetMinMax(connectionPlot.plot)
            connectionPlot.plot.clearData()
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

    Item {
        id: plotLayer
        anchors.fill: parent
    }

    Text {
        anchors {
            right: parent.right
            top: parent.top
            margins: parent.height * 0.04
        }
        font.pixelSize: 12
        text: voltmeterRoot.title
    }

    Text {
        anchors {
            left: parent.left
            top: parent.top
            margins: parent.height * 0.04
        }
        font.pixelSize: 12
        text: maximumValue.toFixed(0)
    }

    Text {
        anchors {
            left: parent.left
            bottom: parent.bottom
            margins: parent.height * 0.04
        }
        font.pixelSize: 12
        text: minimumValue.toFixed(0)
    }

    Item {
        id: resizeRectangle

        Component.onCompleted: {
            resetPosition()
        }

        function resetPosition() {
            x = voltmeterRoot.width - width / 2
            y = voltmeterRoot.height - height / 2
        }

        width: 40
        height: 40
        MouseArea {
            anchors.fill: parent
            drag.target: parent
            onPositionChanged: {
                if(drag.active) {
                    var relativePosition = resizeRectangle.mapToItem(voltmeterRoot, 0, 0)
                    voltmeterRoot.width = relativePosition.x + resizeRectangle.width / 2
                    voltmeterRoot.height = relativePosition.y + resizeRectangle.width / 2
                    resizeRectangle.resetPosition()
                }
            }
        }
    }
}
