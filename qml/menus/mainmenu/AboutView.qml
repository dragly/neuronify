import QtQuick 2.6
import QtQuick.Controls 1.4

import CuteVersioning 1.0

import "../../style"
import "../"


MainMenuPage {
    id: aboutView

    title: "About"

    clip: true

    width: 200
    height: 100

    ScrollView {
        id: aboutFlickable
        anchors {
            fill: parent
            margins: Style.margin
        }

        flickableItem.flickableDirection: Flickable.VerticalFlick
        horizontalScrollBarPolicy: Qt.ScrollBarAlwaysOff

        clip: true
//        contentHeight: aboutText.height

        Text {
            id: aboutText

            onLinkActivated: {
                Qt.openUrlExternally(link)
            }

            width: aboutFlickable.width * 0.95

            font: Style.menu.text.font
            color: Style.menu.text.color

            wrapMode: Text.WrapAtWordBoundaryOrAnywhere
            textFormat: Text.RichText
            text: "
<p>Neuronify is an educational tool meant to give intuition for how neurons and neural
networks behave. You can use it to combine neurons with different connections, just
like the ones we have in our brain, and explore how changes on single cells lead to
behavioral changes in important networks.</p>
<p>We aim to provide a low entry point to simulation-based neuroscience.
Most students won't get the opportunity to create their own neural simulator.
With Neuronify, these students are still able to build up their intuition by
experimenting with neural phenomena.</p>
<p>Neuronify is based on the integrate-and-fire model of neurons. This is one of the
simplest models of neurons that exist. It focuses on the spike timing of a neuron and
ignores the details of the action potential dynamics. These neurons are modelled as
simple RC circuits. When the membrane potential is above a certain threshold, a spike is
generated and the voltage is reset to its resting potential. This spike then signals other
neurons through its synapses.</p>
<p>Version: " + Version.identifier + (Version.dirty ? "*" : "") + "</p>
<p><a href='http://ovilab.net/privacy/'>Privacy policy</a></p>
"
            MouseArea {
                anchors.fill: parent
                cursorShape: parent.hoveredLink ? Qt.PointingHandCursor : Qt.ArrowCursor
                acceptedButtons: Qt.NoButton
            }
        }
    }
}



