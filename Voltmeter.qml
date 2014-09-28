import QtQuick 2.0
import "paths"

Rectangle {
    id: voltmeterRoot

    signal clicked(var voltmeter)

    property bool selected: false
    property var connectionPlots: []
    property var colors: ["#e41a1c", "#377eb8", "#4daf4a", "#984ea3", "#ff7f00", "#a65628", "#f781bf", "#999999"]
    property int currentColor: 0
    property string mode: "voltage"
    property string title: "V"

    property real minimumValue: -100.0
    property real maximumValue: 100.0

    property real lastUpdateTime: Date.now()

    width: 180
    height: 120
    color: "#deebf7"
    border.color: selected ? "#08306b" : "#9ecae1"
    border.width: selected ? 3.0 : 1.0

    function addConnection(connection) {
        if(currentColor > colors.length - 1) {
            currentColor = 0
        }
        var color = colors[currentColor]

        var newList = voltmeterRoot.connectionPlots
        var plotComponent = Qt.createComponent("Plot.qml")
        var plot = plotComponent.createObject(plotLayer, {strokeStyle: color})
        connection.color = color
        resetMinMax(plot)
        newList.push({connection: connection, plot: plot})
        voltmeterRoot.connectionPlots = newList
        currentColor += 1
    }

    function removeConnection(connection) {
        for(var i in connectionPlots) {
            var connectionPlot = connectionPlots[i]
            var connectionOther = connectionPlot.connection
            if(connectionOther === connection) {
                connectionPlots.splice(i, 1)
                connectionPlot.plot.destroy()
                break
            }
        }
    }

    function resetMinMax(plot) {
        plot.minimumValue = minimumValue
        plot.maximumValue = maximumValue
    }

    function stepForward(dt) {
        var currentUpdateTime = Date.now()
        var timeDiff = currentUpdateTime - lastUpdateTime
        if(timeDiff < 36) {
            return
        }

        for(var i in voltmeterRoot.connectionPlots) {
            var connectionPlot = voltmeterRoot.connectionPlots[i]
            var plot = connectionPlot.plot
            var compartment = connectionPlot.connection.itemA
            if(mode === "voltage") {
                plot.addPoint(compartment.voltage)
            } else if(mode === "sodiumCurrent") {
                plot.addPoint(compartment.sodiumCurrent)
            } else if(mode === "potassiumCurrent") {
                plot.addPoint(compartment.potassiumCurrent)
            } else if(mode === "leakCurrent") {
                plot.addPoint(compartment.leakCurrent)
            }
        }
        lastUpdateTime = currentUpdateTime
    }

    onModeChanged: {
        switch(mode) {
        case "voltage":
            minimumValue = -100
            maximumValue = 100
            title = "V"
            break
        case "sodiumCurrent":
            minimumValue = -2e3
            maximumValue = 2e3
            title = "I (Na)"
            break
        case "potassiumCurrent":
            minimumValue = -3e3
            maximumValue = 3e3
            title = "I (K)"
            break
        case "leakCurrent":
            minimumValue = -1e2
            maximumValue = 1e2
            title = "I (L)"
            break
        }
        for(var i in connectionPlots) {
            var connectionPlot = connectionPlots[i]
            resetMinMax(connectionPlot.plot)
            connectionPlot.plot.clearData()
        }
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

//    Timer {
//        id: plotTimer
//        interval: 24
//        running: true
//        repeat: true
//        onTriggered: {

//        }
//    }

    MouseArea {
        anchors.fill: parent
        drag.target: parent
        propagateComposedEvents: true
        onClicked: {
            voltmeterRoot.clicked(voltmeterRoot)
        }
    }

    Item {
        id: resizeRectangle
//        color: "#9ecae1"

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
