import QtQuick 2.0
import "paths"

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

        MouseArea {
            anchors.centerIn: parent
            propagateComposedEvents: true
            width: Math.max(40, Math.min(parent.width, parent.height))
            height: Math.max(40, Math.min(parent.width, parent.height))

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
