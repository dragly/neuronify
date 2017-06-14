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
    filename: "sensors/TouchSensor.qml"
    square: true
    name: "Touch activator"


    property bool sensing: false
    property real sensingCurrentOutput: 100

    property var connections: []

    useDefaultMouseHandling: false
    canReceiveConnections: false

    width: 100
    height: 100
    color: sensorRoot.sensing ? "#80e5ff" : "#dd5900"

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
        source: "qrc:/images/generators/touch_sensor.png"
    }



    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true

        onPressed: {
            sensorRoot.sensing = true
            sensorRoot.clicked(sensorRoot, mouse)
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
        color: "#dd5000"
        connectorColor: "#dd5000"
        z: -1
    }

    MoveHandle {
        id: moveHandle
        opacity: {
            var os = Qt.platform.os
            if(os === "android" || os == "ios") {
                return 1.0
            }
            return 0.0
        }

        states: State {
            name: "hovered"
            when: mouseArea.containsMouse || moveHandle.containsMouse
            PropertyChanges {
                target: moveHandle
                opacity: 1.0
            }
        }

        transitions: Transition {
            NumberAnimation {
                properties: "opacity"
                duration: 240
                easing.type: Easing.InOutQuad
            }
        }
    }

    Keys.onPressed: {
        if(event.key === Qt.Key_Space) {
            sensorRoot.sensing = true;
        }
    }

}

