import QtQuick 2.5
import QtQuick.Controls 2.1
import QtQuick.Controls.Material 2.1
import QtQuick.Dialogs 1.0
import QtQuick.Layouts 1.3
import QtQuick.Particles 2.0
import QtQuick.Window 2.1

import QtCharts 2.1
import QtMultimedia 5.5
import Qt.labs.settings 1.0
import Qt.labs.platform 1.0

import Neuronify 1.0
import CuteVersioning 1.0
import QtGraphicalEffects 1.0

import "qrc:/qml/backend"
import "qrc:/qml/controls"
import "qrc:/qml/hud"
import "qrc:/qml/io"
import "qrc:/qml/menus/filemenu"

import "qrc:/qml/tools"
import "qrc:/qml/store"
import "qrc:/qml/style"
import "qrc:/qml/ui"

Flickable {
    contentHeight: aboutColumn.height + 64
    clip: true

    flickableDirection: Flickable.VerticalFlick
    ScrollBar.vertical: ScrollBar { policy: ScrollBar.AlwaysOn }

    Column {
        id: aboutColumn
        anchors {
            left: parent.left
            right: parent.right
        }

        Label {
            anchors {
                left: parent.left
                right: parent.right
            }
            onLinkActivated: Qt.openUrlExternally(link)
            linkColor: "white"

            wrapMode: Label.WrapAtWordBoundaryOrAnywhere
            text: "
<p><b>Privacy policy</b></p>

<p>See <a href='http://ovilab.net/privacy/'>ovilab.net/privacy</a> for our updated privacy policy.</p>
"
        }

        Label {
            anchors {
                left: parent.left
                right: parent.right
            }

            wrapMode: Label.WrapAtWordBoundaryOrAnywhere
            text: "
<p><b>About Neuronify</b></p>

<p>Neuronify is an educational tool meant to create intuition for how neurons and neural networks behave. You can use it to combine neurons with different connections, just like the ones we have in our brain, and explore how changes on single cells lead to behavioral changes in important networks.</p>

<p>To build and explore neural networks, you drag neurons and measurement devices onto the screen. In addition, the app comes with several ready-made simulations for inspiration.</p>

<p>We aim to provide a low entry point to simulation-based neuroscience. Most students won't get the opportunity to create their own neural simulator. With Neuronify, these students are still able to build up their intuition by experimenting with neural phenomena.</p>

<p>Neuronify is based on an integrate-and-fire model of neurons. This is one of the simplest models of neurons that exist. It focuses on the spike timing of a neuron and ignores the details of the action potential dynamics. These neurons are modeled as simple RC circuits. When the membrane potential is above a certain threshold, a spike is generated and the voltage is reset to its resting potential. This spike then signals other neurons through its synapses.</p>
"
        }
    }
}
