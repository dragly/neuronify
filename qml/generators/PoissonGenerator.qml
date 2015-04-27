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

    objectName: "neuron"
    fileName: "generators/PoissonGenerator.qml"

    width: parent.width * 0.015
    height: width
    color: "#55d400"

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

            Text {
                text: "Stimulation output: " + engine.fireOutput.toFixed(1)
            }
            CheckBox {
                id: inhibitoryCheckbox
                text: "Inhibitory"
                checked: (engine.fireOutput < 0.0)
            }
            Slider {
                id: fireOutputSlider
                minimumValue: 0.0
                maximumValue: 4.0
                stepSize: 0.1
                tickmarksEnabled: false
                value: Math.abs(engine.fireOutput)
            }
            Binding {
                target: engine
                property: "fireOutput"
                value: (inhibitoryCheckbox.checked ? -1.0 : 1.0) * fireOutputSlider.value
            }
        }
    }

    Image {
        anchors.fill: parent

        source: "qrc:/images/generators/poisson_generator.png"
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
