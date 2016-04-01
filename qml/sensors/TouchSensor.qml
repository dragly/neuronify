import QtQuick 2.0

import Neuronify 1.0
import QtGraphicalEffects 1.0
import "../paths"
import "../hud"
import ".."

/*!
\qmltype TouchSensor
\inqmlmodule Neuronify
\ingroup neuronify-sensors
\brief Registers touch (or mouse) events and converts them into a current
       that may be injected into neurons.

The touch sensor uses the mouse input on a desktop computer or the touch screen
of a mobile device.
This allows the user to give input to the neural network.
The input is used to generate a constant current injected into the attached
neurons.
*/

Node {
    id: sensorRoot
    objectName: "touchSensor"
    fileName: "sensors/TouchSensor.qml"
    square: true

    property bool sensing: false
    property real sensingCurrentOutput: 100

    property var dropFunction
    property var connections: []

    useDefaultMouseHandling: false
    canReceiveConnections: false

    width: 100
    height: 100
    color: sensorRoot.sensing ? "#80e5ff" : "#0088aa"

    engine: NodeEngine {
        onStepped: {
            if(sensing) {
                engine.fire()
                sensing = false
            }
        }
    }


    Component.onCompleted: {
        dropFunction = simulator.createConnectionToPoint
    }


    onEdgeAdded: {
        connections.push(edge)
    }

    onEdgeRemoved: {
        connections.splice(connections.indexOf(edge), 1)
    }

    Image {
        id: touchImage
        anchors.fill: parent
        fillMode: Image.PreserveAspectFit
        source: "qrc:/images/sensors/touch_sensor.png"
    }



    MouseArea {
        id: mouseRoot
        anchors.fill: parent

        onPressed: {
            sensorRoot.sensing = true
            sensorRoot.clicked(sensorRoot, mouse)
        }

        onReleased: {
            sensorRoot.sensing = false
        }

        onExited: {
            sensorRoot.sensing =  false
        }
    }


    Connector{
        curveColor: "#0088aa"
        connectorColor: "#0088aa"
        visible: sensorRoot.selected
        z: -1
    }

    Rectangle {
        anchors {
            horizontalCenter: parent.left
            verticalCenter: parent.top
        }
        width: parent.height / 3
        height: width
        radius: width / 2
        color: "#c6dbef"
        border.width: width * 0.1
        border.color: "#f7fbff"

        Image {
            anchors.fill: parent
            anchors.margins: parent.width * 0.1
            source: "qrc:/images/transform-move.png"
            smooth: true
            antialiasing: true
        }

        MouseArea {
            anchors.fill: parent
            drag.target: sensorRoot
            onPressed: {
                sensorRoot.dragging = true
                dragStarted(sensorRoot)
            }

            onClicked: {
                sensorRoot.clicked(sensorRoot, mouse)
            }

            onReleased: {
                sensorRoot.dragging = false
            }
        }
    }

}

