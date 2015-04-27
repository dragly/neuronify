import QtQuick 2.0
import QtQuick.Controls 1.0
import QtMultimedia 5.4
import Neuronify 1.0

import "../paths"
import "../hud"
import "../controls"
import ".."

Node {
    id: root
    objectName: "retina"
    fileName: "sensors/Retina.qml"

    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property VideoSurface videoSurface: null;

    width:20
    height: 20

    dumpableProperties: [
        "x",
        "y"
    ]

    ReceptiveField{
        id:recField
        nPixelsX : 10
        nPixelsY : 10
        receptiveFieldType: ReceptiveField.OffRightRF
    }

    engine: RetinaEngine {
        id: retinaEngine
        receptiveField: recField
        videoSurface: root.videoSurface
    }

    controls: Component {
        Column {
            anchors.fill: parent

            Text {
                text: "X resolution: " + recField.nPixelsX.toFixed(1)
            }
            BoundSlider {
                minimumValue: 10
                maximumValue: 300
                target: recField
                property: "nPixelsX"
            }

            Text {
                text: "Y resolution: " + recField.nPixelsY.toFixed(1)
            }
            BoundSlider {
                minimumValue: 10
                maximumValue: 300
                target: recField
                property: "nPixelsY"
            }
        }
    }

    RetinaPainter {
        id: retinaPainter
        visible: Qt.platform.os !== "android"
        enabled: Qt.platform.os !== "android"
        width: 100
        height: 100
        retinaEngine: retinaEngine

//        MouseArea {
//            anchors.fill: parent
//            drag.target: root
//        }
    }

    Image {
        source: "qrc:/images/sensors/eye.png"
        smooth: true
        antialiasing: true
        anchors.fill: parent
        fillMode: Image.PreserveAspectFit
    }

    Connector {
        visible: root.selected
        onDropped: {
            root.droppedConnector(root, connector)
        }
    }
}

