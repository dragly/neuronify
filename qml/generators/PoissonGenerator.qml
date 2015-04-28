import QtQuick 2.0
import QtQuick.Controls 1.0

import Neuronify 1.0

import "../paths"
import "../hud"
import "../controls"
import ".."

Node {
    id: root

    property point connectionPoint: Qt.point(x + width / 2, y + height / 2)

    objectName: "poissonGenerator"
    fileName: "generators/PoissonGenerator.qml"

    width: parent.width * 0.015
    height: width
    color: inhibitory ? "#d45500" : "#55d400"
    readonly property bool inhibitory: root.engine.fireOutput < 0.0
    property url imageSource: "qrc:/images/generators/poisson_generator.png"
    property url inhibitoryImageSource: "qrc:/images/generators/poisson_generator_inhibitory.png"

    engine: NodeEngine {
        id: engine
        property real rate: 1.0
        fireOutput: 1.0

        onStepped: {
            var shouldFire = (Math.random() < rate*dt)
            if(shouldFire) {
                fire()
                overlayAnimation.restart()
            }
        }
    }

    controls: Component {
        Column {
            anchors.fill: parent

            Text {
                text: "Firing rate: " + engine.rate.toFixed(1) + " s⁻¹"
            }
            BoundSlider {
                minimumValue: 0.0
                maximumValue: 5.0
                target: engine
                property: "rate"
            }
            FireOutputControl {
                target: engine
            }
        }
    }

    Image {
        anchors.fill: parent

        source: inhibitory ? inhibitoryImageSource : imageSource
        smooth: true
        antialiasing: true
        fillMode: Image.PreserveAspectFit
    }

    Image {
        id: overlay
        anchors.fill: parent

        source: "qrc:/images/generators/generator_overlay.png"
        smooth: true
        antialiasing: true
        fillMode: Image.PreserveAspectFit
        opacity: 0
    }

    NumberAnimation {
        id: overlayAnimation
        target: overlay
        property: "opacity"
        from: 0.5
        to: 0
        duration: 200
        easing.type: Easing.OutQuad
    }

    Connector {
        visible: root.selected
        onDropped: {
            root.droppedConnector(root, connector)
        }
    }
}
