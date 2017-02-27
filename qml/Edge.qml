import QtQuick 2.0
import QtQuick.Particles 2.0

import Neuronify 1.0

import "paths"
import "hud"

EdgeBase {
    id: connectionRoot
    signal clicked(var connection)

    property real playbackSpeed
    property string objectName: "edge"
    property bool isEdge: true
    property string filename: "Edge.qml"
    property bool selected: false
    property bool valid: (itemA && itemB) ? true : false
    property color color: valid ? itemA.color : "white"
    property real diffx: valid ? itemA.connectionPoint.x - itemB.connectionPoint.x : 0
    property real diffy: valid ? itemA.connectionPoint.y - itemB.connectionPoint.y : 0
    property real length: Math.sqrt(diffx*diffx + diffy*diffy)
    property real angle: Math.atan2(diffy, diffx)*180/Math.PI
    property real cx: valid ? intersectX(): 0
    property real cy: valid ? intersectY(): 0
    property color _internalColor: connectionRoot.selected ? "#08306b" : connectionRoot.color
    property Component controls: Component {
        Item {}
    }
    readonly property alias startPoint: sCurve.startPoint
    readonly property alias endPoint: sCurve.endPoint

    savedProperties: [
        PropertyGroup {
            property alias filename: connectionRoot.filename
            property alias engine: connectionRoot.engine
        }
    ]

    Component.onCompleted: {
        resetProperties();
        resetDynamics();
    }

    function intersectX() {
        var x

        var dx = Math.abs(diffx)
        var dy = Math.abs(diffy)


        if (!itemB.square) {
            var offset = curved ? 10 * Math.cos((angle + 90) * Math.PI / 180) : 0.0
            return itemB.connectionPoint.x + (connectionSpot.width + itemB.radius) * diffx / length + offset
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
            var offset = curved ? 10 * Math.sin((angle + 90) * Math.PI / 180) : 0.0
            return itemB.connectionPoint.y + (connectionSpot.width + itemB.radius) * diffy / length + offset
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

    Item {
        property real offsetFactor: curved ? 1.8 : 1.0
        property real xOffset: height / 2 * Math.sin(rotation * Math.PI / 180)
        property real yOffset: height / 2 * Math.cos(rotation * Math.PI / 180)

        x: sCurve.startPoint.x + offsetFactor * xOffset
        y: sCurve.startPoint.y - offsetFactor * yOffset

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
                connectionRoot.clicked(connectionRoot)
            }
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


            var x = curved ? length*Math.cos((angle + 90) * Math.PI / 180) : 0.0

            return x_0 + x
        }

        function calculateControlPointY() {
            var dy = cy - startPoint.y
            var length = 20

            var y_0 = startPoint.y + dy/2.
            var y = curved ? length*Math.sin((angle + 90) * Math.PI / 180) : 0.0

            return y_0 + y
        }
    }

    Rectangle {
        id: connectionSpot
        x: cx - width / 2
        y: cy - height / 2
        width: 12
        height: width

        radius: (itemA && itemA.inhibitory) ? width / 2.0 : 0
        rotation: angle + 45
        color: connectionRoot._internalColor

        antialiasing: true
        smooth: true
    }
}
