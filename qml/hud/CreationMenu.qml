import QtQuick 2.0
import QtQuick.Layouts 1.1
import QtQuick.Controls 1.1
import QtGraphicalEffects 1.0

import "../neurons"
import "../style"

Item {
    id: root

    signal droppedEntity(var fileUrl, var properties, var controlParent, var useAutoLayout)
    signal deleteEverything()

    property var blurSource: null
    property bool revealed: false

    width: parent.width * 0.1

    anchors.fill: parent

    onRevealedChanged: {
        var tmpItem = marker.item
        marker.item = null
        marker.item = tmpItem
    }

    Component {
        id: neuronCreators
        CreationList {
            id: itemRow

            CreationItem {
                name: "Passive neuron"
                description: "Neuron with only passive currents."
                source: "qrc:/qml/neurons/PassiveNeuron.qml"
                imageSource: "qrc:/images/creators/neurons/passive.png"
            }

            CreationItem {
                name: "Bursting neuron"
                description: "Neuron that bursts on stimulation."
                source: "qrc:/qml/neurons/BurstNeuron.qml"
                imageSource: "qrc:/images/creators/neurons/burst.png"
            }

            CreationItem {
                name: "Adaptation neuron"
                description: "Neuron passive currents and adaptation on firing."
                source: "qrc:/qml/neurons/AdaptationNeuron.qml"
                imageSource: "qrc:/images/creators/neurons/adaptive.png"
            }
        }
    }

    Component {
        id: inhibitoryNeuronCreators
        CreationList {
            id: itemRow

            CreationItem {
                name: "Passive inhibitory neuron"
                description: "Inhibitory neuron with only passive currents."
                source: "qrc:/qml/neurons/PassiveInhibitoryNeuron.qml"
                imageSource: "qrc:/images/creators/neurons/passive_inhibitory.png"
            }

            CreationItem {
                name: "Bursting inhibitory neuron"
                description: "Neuron that bursts on stimulation."
                source: "qrc:/qml/neurons/BurstNeuron.qml"
                imageSource: "qrc:/images/creators/neurons/burst_inhibitory.png"
            }

            CreationItem {
                name: "Inhibitory adaptation neuron"
                description: "Inhibitory neuron with passive currents and adaptation on firing."
                source: "qrc:/qml/neurons/AdaptationNeuron.qml"
                imageSource: "qrc:/images/creators/neurons/adaptive_inhibitory.png"
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        enabled: root.revealed
        propagateComposedEvents: true
        onPressed: {
            root.revealed = false
            mouse.accepted = false
        }
    }

    Item {
        id: layoutRect

        anchors {
            left: parent.right
            top: parent.top
        }

        width: parent.width
        height: parent.height * 0.5

        MouseArea {
            anchors.fill: parent
            enabled: root.revealed
        }

        Item {
            id: background
            anchors.fill: parent

            ShaderEffectSource {
                id: effectSource
                sourceItem: blurSource
                sourceRect: Qt.rect(layoutRect.x, layoutRect.y, background.width, background.height)
                anchors.fill: parent
            }

            FastBlur {
                anchors.fill: parent
                source: effectSource

                radius: Style.size * 6
            }

            Rectangle {
                anchors.fill: parent
                color: Qt.rgba(1.0, 1.0, 1.0, 0.6)
                border.color: Qt.rgba(0.8, 0.8, 0.8)
                border.width: 2.0
            }
        }

        Image {
            anchors {
                right: parent.right
                top: parent.top
                margins: Style.margin
            }
            width: Style.touchableSize
            height: width

            source: "qrc:/images/back.png"

            rotation: 180

            MouseArea {
                anchors.fill: parent
                onClicked: {
                    root.revealed = false
                }
            }
        }


        Component.onCompleted: {
            marker.item = neuronCategory
        }

        Image {
            id: marker
            property Item item
            source: "qrc:/images/creators/categories/marker.png"

            x: item ? item.mapToItem(parent).x : 0
            y: item ? item.mapToItem(parent).y : 0

            width: item ? item.width : 50
            height: width

            Behavior on x {
                NumberAnimation {
                    duration: 300
                    easing.type: Easing.InOutQuad
                }
            }
        }

        Column {
            anchors {
                fill: parent
                margins: Style.touchableSize * 0.5
            }

            Row {
                height: parent.height / 2
                anchors.horizontalCenter: parent.horizontalCenter

                spacing: Style.touchableSize * 0.5

                Image {
                    id: neuronCategory
                    width: Style.touchableSize
                    height: width
                    source: "qrc:/images/creators/categories/neuron.png"

                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            marker.item = neuronCategory
                            loader.sourceComponent = neuronCreators
                        }
                    }
                }

                Image {
                    id: inhibitoryNeuronCategory
                    source: "qrc:/images/creators/categories/inhibitory_neuron.png"
                    width: Style.touchableSize
                    height: width

                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            marker.item = inhibitoryNeuronCategory
                            loader.sourceComponent = inhibitoryNeuronCreators
                        }
                    }
                }
            }

            Loader {
                id: loader
                anchors.horizontalCenter: parent.horizontalCenter
                height: parent.height / 2
                sourceComponent: neuronCreators
            }
        }

        states: State {
            when: root.revealed
            AnchorChanges {
                target: layoutRect
                anchors.left: root.left
            }
        }

        transitions: Transition {
            AnchorAnimation {
                duration: 400
                easing.type: Easing.InOutQuad
            }
        }
    }
}
