import QtQuick 2.0
import "paths"
import "hud"

import Neuronify 1.0

Node {
    id: voltmeterRoot
    objectName: "voltmeter"
    fileName: "Voltmeter.qml"

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

    dumpableProperties: [
        "x",
        "y"
    ]

    function resetMinMax(plot) {
        plot.minimumValue = minimumValue
        plot.maximumValue = maximumValue
    }

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
                var compartment = connectionPlot.connection.itemA
                if(mode === "voltage") {
                    plot.addPoint(compartment.voltage)
                }
            }
            lastUpdateTime = currentUpdateTime
            timeSinceLastUpdate = 0
        }
    }

    onEdgeAdded: {
        if(currentColor > colors.length - 1) {
            currentColor = 0
        }
        var color = colors[currentColor]

        var newList = voltmeterRoot.connectionPlots
        var plotComponent = Qt.createComponent("Plot.qml")
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
            if(connectionOther === connection) {
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
        border.color: selected ? "#08306b" : "#9ecae1"
        border.width: selected ? 3.0 : 1.0
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
