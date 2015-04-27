import QtQuick 2.0
import QtQuick.Controls 1.0
import QtMultimedia 5.4
import Neuronify 1.0

import "../paths"
import "../hud"
import ".."

Node {
    id: root
    objectName: "neuron"
    fileName: "sensors/Retina.qml"

    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)
    property VideoSurface videoSurface: null;

    width: parent.width * 0.015
    height: width

    dumpableProperties: [
        "x",
        "y"
    ]

    engine: RetinaEngine {
        id: retinaEngine
        videoSurface: root.videoSurface
    }


    RetinaPainter {
        id: retinaPainter
        visible: Qt.platform.os !== "android"
        enabled: Qt.platform.os !== "android"
        width: 100
        height: 100
        retinaEngine: retinaEngine

        MouseArea {
            anchors.fill: parent
            drag.target: parent
        }
    }


    VideoOutput {
        width: 100
        height: 100
        source: !retinaPainter.enabled ? camera : null
        visible: !retinaPainter.visible
        enabled: !retinaPainter.enabled

        MouseArea {
            anchors.fill: parent
            drag.target: parent
        }
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

