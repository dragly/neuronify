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
    id: root

    signal itemClicked(var simulationData)

    contentHeight: column.height + 64
    clip: true
    
    flickableDirection: Flickable.VerticalFlick
    ScrollBar.vertical: ScrollBar {}

    Component.onCompleted: {
        refresh()
    }

    function refresh() {
        communityProgressBar.processCount += 1
        Firebase.get('simulations.json', function(response) {
            communityProgressBar.processCount -= 1
            console.log("Model", JSON.stringify(response))

            communityRepeater.model = Firebase.createModel(response)
        })
    }

    Column {
        id: column
        anchors {
            left: parent.left
            right: parent.right
        }
        
        spacing: 16
        
        ProgressBar {
            id: communityProgressBar
            
            property int processCount: 0
            
            indeterminate: true
            visible: processCount > 0
        }

        Label {
            anchors {
                left: parent.left
                right: parent.right
            }

            wrapMode: Label.WrapAtWordBoundaryOrAnywhere
            text: "<p>Simulations in this view are shared by other Neuronify users. " +
                  "Please report any broken simulations, problems, or abuse of this feature to ovilab.net@gmail.com</p>"
        }

        DownloadManager {
            id: downloadManager
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
                    property var objectData

                    Component.onCompleted: {
                        communityProgressBar.processCount += 1
                        objectData = modelData
                        name = modelData.name
                        description = modelData.description
                        Firebase.cachedDownload(
                                    modelData.screenshot,
                                    function (localFileName) {
                                        communityProgressBar.processCount -= 1
                                        imageUrl = localFileName
                                    })
                    }

                    onClicked: {
                        itemClicked(objectData)
                    }
                }
            }
        }
    }
}
