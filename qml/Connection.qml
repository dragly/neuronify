import QtQuick 2.0
import "paths"
import "hud"
import Neuronify 1.0

Edge {
    id: connectionRoot
    signal clicked(var connection)

    property string objectName: "connection"
    property bool selected: false
    property bool valid: (itemA && itemB) ? true : false
    property real conductance: 1.0
    property color color: valid ? itemA.color : "white"
    property real diffx: valid ? itemA.connectionPoint.x - itemB.connectionPoint.x + 10*curved: 0
    property real diffy: valid ? itemA.connectionPoint.y - itemB.connectionPoint.y + 10*curved: 0
    property real length: Math.sqrt(diffx*diffx + diffy*diffy)
    property real angle: Math.atan(diffy/diffx)*180/Math.PI
    property real cx: valid ? intersectX(): 0
    property real cy: valid ? intersectY(): 0
    property color _internalColor: connectionRoot.selected ? "#08306b" : connectionRoot.color
    property var customDump
    property Component controls: Component {
        Item {}
    }

    function intersectX() {
        var x

        var dx = Math.abs(diffx)
        var dy = Math.abs(diffy)


        if (!itemB.square) {
            return itemB.connectionPoint.x + (connectionSpot.width + itemB.radius) * diffx / length
        }

        if (diffx <= 0 && diffy >= 0) {
            if (dx >= dy) {
                x = itemB.connectionPoint.x - (itemB.width/2. + connectionSpot.width)

            } else {
                x = itemB.connectionPoint.x - (itemB.width/2. + connectionSpot.width)*dx/dy
            }

        } else if (diffx >= 0 && diffy >= 0) {
            if (dx >= dy) {
                x = itemB.connectionPoint.x + (itemB.width/2. + connectionSpot.width)
            } else {
                x = itemB.connectionPoint.x + (itemB.width/2. + connectionSpot.width)*dx/dy
            }
        } else if (diffx >= 0 && diffy <= 0) {
            if (dx >= dy) {
                x = itemB.connectionPoint.x + (itemB.width/2. + connectionSpot.width)
            } else {
                x = itemB.connectionPoint.x + (itemB.width/2. + connectionSpot.width)*dx/dy
            }

        } else {
            if (dx >= dy) {
                x = itemB.connectionPoint.x - (itemB.width/2. + connectionSpot.width)
            } else {
                x = itemB.connectionPoint.x - (itemB.width/2. + connectionSpot.width)*dx/dy
            }
        }

        return x
    }


    function intersectY() {
        var y

        var dx = Math.abs(diffx)
        var dy = Math.abs(diffy)

        if (!itemB.square) {
            return itemB.connectionPoint.y + (connectionSpot.width + itemB.radius) * diffy / length
        }


        if (diffx <= 0 && diffy >= 0) {
            if (dx >= dy) {
                y = itemB.connectionPoint.y + (itemB.height/2 + connectionSpot.width)*dy/dx
            } else {
                y = itemB.connectionPoint.y + (itemB.height/2. + connectionSpot.width)
            }

        } else if (diffx >= 0 && diffy >= 0) {
            if (dx >= dy) {
                y = itemB.connectionPoint.y + (itemB.height/2 + connectionSpot.width)*dy/dx
            } else {
                y = itemB.connectionPoint.y + (itemB.height/2. + connectionSpot.width)
            }

        } else if (diffx >= 0 && diffy <= 0) {
            if (dx >= dy) {
                y = itemB.connectionPoint.y - (itemB.height/2 + connectionSpot.width)*dy/dx
            } else {
                y = itemB.connectionPoint.y - (itemB.height/2. + connectionSpot.width)
            }

        } else {
            if (dx >= dy) {
                y = itemB.connectionPoint.y - (itemB.height/2 + connectionSpot.width)*dy/dx
            } else {
                y = itemB.connectionPoint.y - (itemB.height/2. + connectionSpot.width)
            }


        }

        return y
    }


    function dump(index, graphEngine) {
        if(customDump) {
            return customDump(index, graphEngine)
        }
        var itemAEntityIndex = graphEngine.nodeIndex(itemA)
        if(itemAEntityIndex === -1) {
            console.error("Could not find index of node " + itemA + " in GraphEngine! Aborting dump!")
            return ""
        }
        var itemBEntityIndex = graphEngine.nodeIndex(itemB)
        if(itemBEntityIndex === -1) {
            console.error("Could not find index of node " + itemB + " in GraphEngine! Aborting dump!")
            return ""
        }
        return {
            "from": itemAEntityIndex,
            "to": itemBEntityIndex
        }
    }

    BezierCurve {
        id: sCurve
        color: connectionRoot._internalColor
        startPoint: itemA ? Qt.point(itemA.connectionPoint.x, itemA.connectionPoint.y) : Qt.point(0,0)
        endPoint: Qt.point(cx, cy)

        controlPoint1: Qt.point(calculateControlPointX(), calculateControlPointY())
        controlPoint2: Qt.point(calculateControlPointX(), calculateControlPointY())

        function calculateControlPointX() {
            var dx = cx - startPoint.x
            var length = 20

            var x_0 = startPoint.x + dx/2.


            var x = curved*length*Math.cos((angle + 90) * Math.PI / 180)

            return x_0 + x
        }

        function calculateControlPointY() {
            var dy = cy - startPoint.y
            var length = 20

            var y_0 = startPoint.y + dy/2.
            var y = curved*length*Math.sin((angle + 90) * Math.PI / 180)

            return y_0 + y
        }
    }




    Item {
        x: sCurve.startPoint.x + height / 2 * Math.sin(rotation * Math.PI / 180)
        y: sCurve.startPoint.y - height / 2 * Math.cos(rotation * Math.PI / 180)

        transformOrigin: Item.TopLeft

        width: Qt.vector2d(sCurve.endPoint.x - sCurve.startPoint.x,
                           sCurve.endPoint.y - sCurve.startPoint.y).length()
        height: 40

        rotation: Math.atan2(sCurve.endPoint.y - sCurve.startPoint.y,
                             sCurve.endPoint.x - sCurve.startPoint.x) / Math.PI * 180

        MouseArea {
            anchors.fill: parent
            propagateComposedEvents: true

            onClicked: {
                if(connectionRoot.selected) {
                    mouse.accepted = false
                }

                connectionRoot.clicked(connectionRoot)
            }
        }
    }

    Rectangle {
        id: connectionSpot
        x: cx - width / 2
        y: cy - height / 2
        width: 10
        height: width

        radius: (itemA && itemA.engine) ? (itemA.engine.fireOutput > 0 ?  0 : width / 2.0) : width / 2.0;
        rotation: angle + 45
        color: connectionRoot._internalColor

        antialiasing: true
        smooth: true
    }
}
