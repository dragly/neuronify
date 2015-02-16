import QtQuick 2.0
import "paths"
import "hud"

Item {
    id: connectionRoot
    signal clicked(var connection)
    property bool selected: false
    property var itemA
    property var itemB
    property real conductance: 1.0
    property color color: itemA.color
    property real diffx: itemA.connectionPoint.x - itemB.connectionPoint.x
    property real diffy: itemA.connectionPoint.y - itemB.connectionPoint.y
    property real length: Math.sqrt(diffx*diffx + diffy*diffy)
    property real cx: itemB.connectionPoint.x + (connectionSpot.width + itemB.radius) * diffx / length
    property real cy: itemB.connectionPoint.y + (connectionSpot.width + itemB.radius) * diffy / length
    property color _internalColor: connectionRoot.selected ? "#08306b" : connectionRoot.color
    property Component controls: Component {
        ConnectionControls {
            connection: connectionRoot
        }
    }

    function otherCompartment(currentCompartment) {
        if(currentCompartment === itemA) {
            return itemB
        } else {
            return itemA
        }
    }

    Line {
        id: sCurve
        color: connectionRoot._internalColor
        startPoint: Qt.point(itemA.connectionPoint.x, itemA.connectionPoint.y)
        endPoint: Qt.point(cx, cy)
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
        width: 6
        height: width
        radius: width / 2.0
        color: connectionRoot._internalColor
    }
}
