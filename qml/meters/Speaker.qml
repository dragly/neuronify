import QtQuick 2.0
import QtQuick.Controls 1.0

import Neuronify 1.0

import ".."
import "../controls"

Node {
    objectName: "Speaker"
    fileName: "meters/Speaker.qml"

    width: 62
    height: 62

    engine: NodeEngine {
        onReceivedFire: {
            sound.play()
        }
    }

    controls: Component {
        Column {
            CheckBox {
                id: mutedCheckBox
                text: "Muted"
                checked: sound.muted
            }
            Binding {
                target: sound
                property: "muted"
                value: mutedCheckBox.checked
            }

            Text {
                text: "Volume: " + sound.volume.toFixed(1)
            }
            BoundSlider {
                target: sound
                property: "volume"
                minimumValue: 0.0
                maximumValue: 1.0
            }
        }
    }

    Image {
        anchors.fill: parent
        fillMode: Image.PreserveAspectFit
        antialiasing: true
        smooth: true
        source: "qrc:/images/meters/speaker.png"
    }

    SoundBank {
        id: sound
    }
}

