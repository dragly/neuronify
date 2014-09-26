import QtQuick 2.0
import "paths"

Item {
    id: connectionRoot
    signal clicked(var connection)
    property bool selected: false
    property var sourceCompartment
    property var targetCompartment
    property real conductance: 1.0
    property color color: "#4292c6"

    function otherCompartment(currentCompartment) {
        if(currentCompartment === sourceCompartment) {
            return targetCompartment
        } else {
            return sourceCompartment
        }
    }

    SCurve {
        id: sCurve
        color: connectionRoot.selected ? "#08306b" : connectionRoot.color
        startPoint: Qt.point(sourceCompartment.x + sourceCompartment.width / 2.0, sourceCompartment.y + sourceCompartment.height / 2)
        endPoint: Qt.point(targetCompartment.x + targetCompartment.width / 2.0, targetCompartment.y + targetCompartment.height / 2)

        MouseArea {
            anchors.centerIn: parent
            width: Math.max(40, Math.min(parent.width, parent.height))
            height: Math.max(40, Math.min(parent.width, parent.height))

            onClicked: {
                connectionRoot.clicked(connectionRoot)
            }
        }
    }
}
