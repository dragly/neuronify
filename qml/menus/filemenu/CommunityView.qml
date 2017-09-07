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
    signal runRequested(var simulation)
    signal itemClicked(var simulation, var simulationData)

    contentHeight: column.height + 64
    clip: true
    
    flickableDirection: Flickable.VerticalFlick
    ScrollBar.vertical: ScrollBar {}
    
    Column {
        id: column
        anchors {
            left: parent.left
            right: parent.right
        }
        
        spacing: 16
        
        Component.onCompleted: {
            communityProgressBar.processCount += 1
            Parse.get('Tag?order=-priority', function(response) {
                communityProgressBar.processCount -= 1
                communityTagsRepeater.model = response.results
            })
        }
        
        Label {
            anchors {
                left: parent.left
                right: parent.right
            }
            
            wrapMode: Label.WrapAtWordBoundaryOrAnywhere
            onLinkActivated: Qt.openUrlExternally(link)
            linkColor: "white"
            
            text: "Would you like to see your simulations listed here? " +
                  "Send an e-mail to <a href='mailto:ovilab.net@gmail.com'>ovilab.net@gmail.com</a> " +
                  "to request upload rights."
        }
        
        ProgressBar {
            id: communityProgressBar
            
            property int processCount: 0
            
            indeterminate: true
            visible: processCount > 0
        }
        
        Repeater {
            id: communityTagsRepeater
            
            Column {
                id: tag
                
                anchors {
                    left: parent.left
                    right: parent.right
                }
                spacing: 16
                
                Component.onCompleted: {
                    communityProgressBar.processCount += 1
                    Parse.get('Simulation?where={"tags":{"__type":"Pointer","className":"Tag","objectId":"' + modelData.objectId + '"}}', function(response) {
                        communityProgressBar.processCount -= 1
                        communityRepeater.model = response.results
                    })
                }
                
                Label {
                    font.pixelSize: 24
                    text: modelData.name
                }
                
                FolderListModel {
                    id: communityFolderModel
                    folder: StandardPaths.writableLocation(StandardPaths.AppDataLocation) + "/community"
                    showFiles: false
                    showDirs: true
                    showOnlyReadable: true
                }
                
                Component {
                    id: simulationComponent
                    StoreSimulation {
                        downloadManager: _downloadManager // TODO is this needed anymore?
                        onRunClicked: {
                            //                                        loadRequested(fileUrl)
                            //                                        openRequested(fileUrl)
                            runRequested(simulation)
                        }
                    }
                }
                
                Flow {
                    id: flow
                    anchors {
                        left: parent.left
                        right: parent.right
                    }
                    
                    spacing: 16
                    
                    Repeater {
                        id: communityRepeater
                        delegate: StoreItem {
                            name: modelData.name
                            description: modelData.description
                            imageUrl: modelData.screenshot.url
                            onClicked: {
                                itemClicked(simulationComponent, modelData)
                            }
                        }
                    }
                }
            }
        }
    }
}
