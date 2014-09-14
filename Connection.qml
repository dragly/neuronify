import QtQuick 2.0
import "paths"

Item {
    property var sourceCompartment
    property var targetCompartment
    property real axialConductance: 1.0

    function otherCompartment(currentCompartment) {
        if(currentCompartment === sourceCompartment) {
            return targetCompartment
        } else {
            return sourceCompartment
        }
    }

    SCurve {
        id: sCurve

        startPoint: Qt.point(sourceCompartment.x + sourceCompartment.width / 2.0, sourceCompartment.y + sourceCompartment.height / 2)
        endPoint: Qt.point(targetCompartment.x + targetCompartment.width / 2.0, targetCompartment.y + targetCompartment.height / 2)
    }
}
