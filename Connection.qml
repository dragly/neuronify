import QtQuick 2.0
import "paths"

Item {
    id: connectionRoot
    signal clicked(var connection)
    property bool selected: false
    property var itemA
    property var itemB
    property real conductance: 1.0
    property color color: "#4292c6"

    function otherCompartment(currentCompartment) {
        if(currentCompartment === itemA) {
            return itemB
        } else {
            return itemA
        }
    }

    SCurve {
        id: sCurve
        color: connectionRoot.selected ? "#08306b" : connectionRoot.color
        startPoint: Qt.point(itemA.connectionPoint.x, itemA.connectionPoint.y)
        endPoint: Qt.point(itemB.connectionPoint.x, itemB.connectionPoint.y)

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
}
