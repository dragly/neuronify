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
import Qt.labs.folderlistmodel 2.1
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
    signal loadRequested(var filename)

    contentHeight: newColumn.height + 64
    clip: true
    
    flickableDirection: Flickable.VerticalFlick
    ScrollBar.vertical: ScrollBar { policy: ScrollBar.AlwaysOn }
    Column {
        id: newColumn
        anchors {
            left: parent.left
            right: parent.right
            rightMargin: 16
        }
        
        spacing: 32
        
        StoreItem {
            name: "New simulation"
            description: "A blank canvas."
            onClicked: {
                loadRequested("qrc:/simulations/empty/empty.nfy")
            }
        }
        
        // TODO replace with database
        Repeater {
            model: [
                {
                    name: "Tutorial",
                    simulations: [
                        "qrc:/simulations/tutorial/tutorial_1_intro",
                        "qrc:/simulations/tutorial/tutorial_2_circuits",
                        "qrc:/simulations/tutorial/tutorial_3_creation",
                    ]
                },
                {
                    name: "Neuronify Items",
                    simulations: [
                        "qrc:/simulations/items/neurons/leaky",
                        "qrc:/simulations/items/neurons/inhibitory",
                        "qrc:/simulations/items/neurons/adaptation",
                        "qrc:/simulations/items/visualInput",
                        "qrc:/simulations/items/generators",
                        "qrc:/simulations/items/frPlot",
                        
                    ]
                },
                {
                    name: "Miscellaneous",
                    simulations: [
                        "qrc:/simulations/mix/lateral_inhibition",
                        "qrc:/simulations/mix/recurrent_inhibition",
                        "qrc:/simulations/mix/reciprocal_inhibition",
                        "qrc:/simulations/mix/disinhibition",
                        "qrc:/simulations/mix/rythm_transformation",
                        "qrc:/simulations/mix/prolonged_activity",
                    ]
                },
                {
                    name: "Textbook Examples",
                    simulations: [
                        "qrc:/simulations/mix/lateral_inhibition_1",
                        "qrc:/simulations/mix/lateral_inhibition_2",
                        "qrc:/simulations/mix/input_summation",
                        "qrc:/simulations/sterratt/if_response",
                        "qrc:/simulations/sterratt/refractory_period",
                    ]
                },
            ]
            
            Column {
                
                anchors {
                    left: parent.left
                    right: parent.right
                }
                
                spacing: 16
                
                Label {
                    font.pixelSize: 24
                    text: modelData.name
                }
                
                Flow {
                    anchors {
                        left: parent.left
                        right: parent.right
                    }
                    spacing: 32
                    Repeater {
                        model: modelData.simulations
                        StoreItem {
                            SimulationLoader {
                                id: loader
                                folder: modelData
                            }
                            
                            name: loader.item.name
                            description: loader.item.description
                            imageUrl: loader.item.screenshotSource
                            
                            onClicked: {
                                loadRequested(loader.item.stateSource)
                            }
                        }
                    }
                }
            }
        }
    }
}
