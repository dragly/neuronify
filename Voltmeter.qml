import QtQuick 2.0
import "paths"

Rectangle {
    id: voltmeterRoot

    property var compartmentPlots: []
//    signal droppedConnectionCreator(var compartment, var connectionCreator)
    property var colors: ["#e41a1c", "#377eb8", "#4daf4a", "#984ea3", "#ff7f00", "#a65628", "#f781bf", "#999999"]
    property int currentColor: 0

    width: 100
    height: 100
    color: "#f7fbff"
    border.color: "#deebf7"
    border.width: 1.0

    function addCompartment(compartment) {
        if(currentColor > colors.length - 1) {
            currentColor = 0
        }
        var color = colors[currentColor]

        var newCompartments = voltmeterRoot.compartmentPlots
        var plotComponent = Qt.createComponent("Plot.qml")
        var plot = plotComponent.createObject(voltmeterRoot, {strokeStyle: color})
        newCompartments.push({compartment: compartment, plot: plot})
        voltmeterRoot.compartmentPlots = newCompartments
        currentColor += 1
    }

    function removeCompartment(compartment) {
        for(var i in compartmentPlots) {
            var compartmentPlot = compartmentPlots[i]
            var compartmentOther = compartmentPlot.compartment
            if(compartmentOther === compartment) {
                compartmentPlots.splice(i, 1)
                compartmentPlot.plot.destroy()
                break
            }
        }
    }

    Timer {
        id: plotTimer
        interval: 24
        running: true
        repeat: true
        onTriggered: {
            for(var i in voltmeterRoot.compartmentPlots) {
                var compartmentPlot = voltmeterRoot.compartmentPlots[i]
                var plot = compartmentPlot.plot
                var compartment = compartmentPlot.compartment
                plot.addPoint(compartment.voltage)
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        drag.target: parent
        propagateComposedEvents: true
    }

    Rectangle {
        id: resizeRectangle
        color: "#9ecae1"

        Component.onCompleted: {
            resetPosition()
        }

        function resetPosition() {
            x = voltmeterRoot.width - width / 2
            y = voltmeterRoot.height - height / 2
        }

        width: 20
        height: 20
        MouseArea {
            anchors.fill: parent
            drag.target: parent
            onPositionChanged: {
                if(drag.active) {
                    var relativePosition = resizeRectangle.mapToItem(voltmeterRoot, 0, 0)
                    voltmeterRoot.width = relativePosition.x
                    voltmeterRoot.height = relativePosition.y
                    resizeRectangle.resetPosition()
                }
            }
        }
    }
}
