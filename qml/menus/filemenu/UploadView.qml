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

Item {
    id: root
    signal uploadCompleted()
    Column {
        id: uploadColumn
        width: 420
        height: 420
        spacing: 8

        Label {
            text: "Name:"
            //                                    color: Material.color(Material.Grey)
        }

        TextField {
            id: uploadNameField
            anchors {
                left: parent.left
                right: parent.right
            }
            placeholderText: "e.g. 'Lateral inhibition'"
        }

        Label {
            text: "Description:"
        }

        TextField {
            id: uploadDescriptionField
            anchors {
                left: parent.left
                right: parent.right
            }

            placeholderText: "e.g. 'Shows network effects of lateral inhibition.'"
        }

        Label {
            text: "Screenshot preview:"
        }

        ShaderEffectSource {
            readonly property real aspectRatio: neuronify.width / neuronify.height
            width: parent.width * 0.6
            height: width / aspectRatio
            sourceItem: neuronify
        }

        Row {
            anchors {
                right: parent.right
            }
            spacing: 16
            Button {
                id: uploadButton
                Material.theme: Material.Light
                text: "Upload"
                onClicked: {
                    var simulation = {
                        name: uploadNameField.text,
                        description: uploadDescriptionField.text
                    }
                    uploadButton.enabled = false
                    Firebase.post("simulations.json", simulation, function(result) {
                        console.log("Simulation created!")
                        var simulationAddress = "simulations/" + result.name
                        var tempFolder = StandardPaths.writableLocation(StandardPaths.TempLocation)
                        var screenshotFilename = tempFolder + "/screenshot.png"
                        // TODO do not reference "global" items
                        neuronify.saveScreenshot(screenshotFilename, function() {
                            console.log("Saved screenshot for upload")
                            var data = neuronify.fileManager.serializeState()
                            Firebase.upload(simulationAddress + "/simulation.nfy", data, function(simulationFile) {
                                console.log("Uploaded the simulation!")
                                console.log(JSON.stringify(simulationFile))
                                // TODO move download manager here
                                _downloadManager.upload(
                                            screenshotFilename,
                                            Firebase.buildUploadUrl(simulationAddress + "/screenshot.png"),
                                            Firebase.idToken,
                                            function(screenshotResult) {
                                                console.log("Uploaded screenshot!")
                                                console.log(screenshotResult)
                                                var screenshotFile = JSON.parse(screenshotResult)
                                                simulation["simulation"] = simulationFile.name
                                                simulation["screenshot"] = screenshotFile.name
                                                Firebase.put(simulationAddress + ".json", simulation, function(result) {
                                                    console.log("Upload complete!")
                                                    console.log(JSON.stringify(result))
                                                    ToolTip.show("Upload successful!", 2000)
                                                    uploadButton.enabled = true
                                                    root.uploadCompleted()
                                                })
                                            })
                            })
                        })
                    })
                }
            }
        }
    }
}
