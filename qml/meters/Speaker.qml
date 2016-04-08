import QtQuick 2.0
import QtQuick.Controls 1.0

import QtQuick.Controls.Styles 1.0

import Neuronify 1.0

import ".."
import "../controls"
import "../edges"
import "../style"

/*!
\qmltype Speaker
\inqmlmodule Neuronify
\ingroup neuronify-meters
\brief A speaker that emits a sound whenever a connected neuron fires

Neurons can connect to the speaker. When they do, the speaker will emit a sound
whenever the neuron fires an action potential. The sound and volume can be set in
the associated controls Component.
*/

Node {
    id: speaker
    property alias source: soundBank.source

    objectName: "Speaker"
    fileName: "meters/Speaker.qml"

    width: 64
    height: 64

    preferredEdge: MeterEdge {}

    canReceiveConnections: false

    engine: NodeEngine {
        onReceivedFire: {
            soundBank.play()
        }
    }

    controls: Component {
        PropertiesPage {
            property string title: "Speaker"
            width: parent ? parent.width : 100

            Component.onCompleted: {
                for(var i = 0; i < repeater.count; i++) {
                    var item = repeater.itemAt(i)
                    if(Qt.resolvedUrl(item.source) === Qt.resolvedUrl(soundBank.source)) {
                        item.checked = true
                    }
                }
            }


            BoundSlider {
                target: soundBank
                property: "volume"
                minimumValue: 0.0
                maximumValue: 1.0
                text: "Volume"
            }

            CheckBox {
                id: mutedCheckBox
                Text{
                    anchors.left: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Muted"
                    font: Style.control.font
                    color: Style.text.color
                }

                checked: soundBank.muted
            }
            Binding {
                target: soundBank
                property: "muted"
                value: mutedCheckBox.checked
            }
            Button {
                text: "Connect to all neurons"
                onClicked: {
                    var itemA = speaker
                    for (var i in speaker.simulator.graphEngine.nodes){
                        var itemB = graphEngine.nodes[i]
                        if (itemB.objectName.indexOf("neuron") + itemB.objectName.indexOf("Neuron") != -2){
                            if (!speaker.simulator.connectionExists(itemA, itemB)){
                                speaker.simulator.connectEntities(itemA, itemB)
                            }
                        }
                    }
                }
            }

            ExclusiveGroup { id: soundGroup }
            Repeater {
                id: repeater
                model: soundsModel
                RadioButton {
                    property url source: model.source
                    exclusiveGroup: soundGroup

                    Text{
                        anchors.left: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        text:  model.name
                        font: Style.control.font
                        color: Style.text.color
                    }
//                    text: model.name

                    onCheckedChanged: {
                        if(checked) {
                            soundBank.source = model.source
                        }
                    }
                }
            }
        }
    }

    savedProperties: PropertyGroup {
        property alias source: speaker.source
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



    Connector {}
}

