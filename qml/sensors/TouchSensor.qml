import QtQuick 2.0

import Neuronify 1.0
import QtGraphicalEffects 1.0
import ".."
import "../controls"
import "../edges"
import "../hud"
import "../paths"
import "../tools"

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

// TODO add custom synapse for TouchSensor

Node {
    id: sensorRoot
    objectName: "touchSensor"
    fileName: "sensors/TouchSensor.qml"
    square: true

    property bool sensing: false
    property real sensingCurrentOutput: 100

    property var connections: []

    useDefaultMouseHandling: false
    canReceiveConnections: false

    width: 100
    height: 100
    color: sensorRoot.sensing ? "#80e5ff" : "#0088aa"

    preferredEdge: ImmediateFireSynapse {}

    engine: NodeEngine {
        onStepped: {
            if(sensing) {
                engine.fire()
                sensing = false
                overlayAnimation.restart()
            }
        }
    }

    controls: Component {
        PropertiesPage {
             property string title: "Touch sensor"

        }
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


    Image {
        id: overlay
        anchors.fill: parent

        source: "qrc:/images/sensors/touch_sensor_overlay.png"
        smooth: true
        antialiasing: true
        fillMode: Image.PreserveAspectFit
        opacity: 0
    }

    NumberAnimation {
        id: overlayAnimation
        target: overlay
        property: "opacity"
        from: 1.0
        to: 0
        duration: 1000
        easing.type: Easing.OutQuad
    }

    Connector{
        curveColor: "#0088aa"
        connectorColor: "#0088aa"
        visible: sensorRoot.selected
        z: -1
    }

    MoveHandle {
    }

}

