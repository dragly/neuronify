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
    property var neuronBindings: []
    property int numberOfEdges: 0

    objectName: "Loudspeaker"
    filename: "meters/Loudspeaker.qml"
    name: "Speaker"

    width: 64
    height: 64

    preferredEdge: MeterEdge {}

    canReceiveConnections: false

    controls: Component {
        PropertiesPage {
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

//            ConnectMultipleControl {
//                toEnabled: false
//                node: speaker
//            }
        }
    }

    savedProperties: PropertyGroup {
        property alias source: speaker.source
    }

    onEdgeAdded: {
        numberOfEdges +=1
        var neuron = edge.itemB;
        var binding = {
            neuron: neuron,
            playSound: function() {
                soundBank.play();
            }
        }
        neuron.fired.connect(binding.playSound);
        neuronBindings.push(binding);
    }

    onEdgeRemoved: {
        var neuron = edge.itemB;
        var newList = [];
        for(var i in neuronBindings) {
            var binding = neuronBindings[i];
            if(binding.neuron !== neuron) {
                newList.push(binding);
            } else {
                binding.neuron.fired.disconnect(binding.playSound);
            }
        }
        neuronBindings = newList;
        numberOfEdges -=1
    }

    Image {
        anchors.fill: parent
        fillMode: Image.PreserveAspectFit
        antialiasing: true
        smooth: true
        source: "qrc:/images/meters/loudspeaker.png"
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

    Connector {
        color: Style.meter.border.color
    }
}

