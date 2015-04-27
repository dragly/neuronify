import QtQuick 2.0
import QtQuick.Controls 1.0

import Neuronify 1.0

import ".."
import "../controls"

Node {
    property alias source: soundBank.source

    objectName: "Speaker"
    fileName: "meters/Speaker.qml"

    width: 62
    height: 62

    engine: NodeEngine {
        onReceivedFire: {
            soundBank.play()
        }
    }

    controls: Component {
        Column {
            width: parent.width

            Component.onCompleted: {
                for(var i = 0; i < repeater.count; i++) {
                    var item = repeater.itemAt(i)
                    if(Qt.resolvedUrl(item.source) === Qt.resolvedUrl(soundBank.source)) {
                        item.checked = true
                    }
                }
            }

            CheckBox {
                id: mutedCheckBox
                text: "Muted"
                checked: soundBank.muted
            }
            Binding {
                target: soundBank
                property: "muted"
                value: mutedCheckBox.checked
            }

            Text {
                text: "Volume: " + soundBank.volume.toFixed(1)
            }
            BoundSlider {
                target: soundBank
                property: "volume"
                minimumValue: 0.0
                maximumValue: 1.0
            }

            ExclusiveGroup { id: soundGroup }
            Repeater {
                id: repeater
                model: soundsModel
                RadioButton {
                    property url source: model.source
                    exclusiveGroup: soundGroup

                    text: model.name

                    onCheckedChanged: {
                        if(checked) {
                            console.log("Setting source: " + model.source)
                            soundBank.source = model.source
                        }
                    }
                }
            }
        }
    }

    Component.onCompleted: {
        dumpableProperties = dumpableProperties.concat("source")
    }

    Image {
        anchors.fill: parent
        fillMode: Image.PreserveAspectFit
        antialiasing: true
        smooth: true
        source: "qrc:/images/meters/speaker.png"
    }

    ListModel {
        id: soundsModel
        ListElement {
            name: "Drip"
            source: "drip.wav"
        }
        ListElement {
            name: "Sonar"
            source: "sonar.wav"
        }
        ListElement {
            name: "Thump"
            source: "thump.wav"
        }
        ListElement {
            name: "Glass"
            source: "glass.wav"
        }
    }

    SoundBank {
        id: soundBank
        source: "glass.wav"
    }
}

