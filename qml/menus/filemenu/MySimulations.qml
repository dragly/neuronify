import QtQuick 2.9
import QtQuick.Controls 2.2
import QtQuick.Controls.Material 2.2
import Neuronify 1.0

import "qrc:/qml/backend"
import "qrc:/qml/store"

Item {
    id: root

    signal uploadClicked()
    signal itemClicked(var simulationData)

    Component.onCompleted: {
        progressBar.processCount += 1
        Firebase.get('simulations.json?orderBy="creator"&equalTo="' + Firebase.userId + '"', function(response) {
            progressBar.processCount -= 1
            communityRepeater.model = Firebase.createModel(response)
        })
    }

    Column {
        id: tag

        anchors {
            left: parent.left
            right: parent.right
        }
        spacing: 16

        Button {
            Material.theme: Material.Light
            text: "Upload"
            onClicked: {
                uploadClicked()
            }
        }

        ProgressBar {
            id: progressBar

            property int processCount: 0

            indeterminate: true
            visible: processCount > 0
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
                    imageUrl: modelData.screenshot
                    onClicked: {
                        itemClicked(modelData)
                    }
                }
            }
        }
    }
}
