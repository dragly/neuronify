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

    signal uploadCompleted()

    contentHeight: uploadColumn.height + 64
    clip: true

    ScrollBar.vertical: ScrollBar { policy: ScrollBar.AlwaysOn }
    flickableDirection: Flickable.VerticalFlick

    Column {
        id: uploadColumn
        width: 420

        Label {
            anchors {
                left: parent.left
                right: parent.right
            }

            wrapMode: Label.WrapAtWordBoundaryOrAnywhere
            onLinkActivated: Qt.openUrlExternally(link)
            text: "
<p>Terms and conditions</p>
<p>By uploading a simulation,
including any assets such as the automatically generated screenshot,
you confirm that you are the copyright holder of this work
and irrevocably grant anyone the right to use this work under the
Creative Commons Attribution ShareAlike 4.0 license
(<a href='https://creativecommons.org/licenses/by-sa/4.0'>legal code</a>).</p>

<p>NOTE: This is an experimental feature of Neuronify.
We cannot guarantee the availability of this feature in the future.</p>
"
        }

        CheckBox {
            id: agreeCheckbox
            text: "I agree to the terms and conditions"
        }

        Column {
            enabled: agreeCheckbox.checked
            width: parent.width
            spacing: parent.spacing
            Label {
                text: "Name:"
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
                            description: uploadDescriptionField.text,
                            uid: Firebase.userId
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
                                var data = JSON.stringify(neuronify.fileManager.serializeState())
                                Firebase.uploadText(simulationAddress + "/simulation.nfy", data, function(simulationResult) {
                                    var simulationFile = JSON.parse(simulationResult)
                                    console.log("Uploaded the simulation!", simulationFile)
                                    Firebase.upload(
                                                simulationAddress + "/screenshot.png",
                                                screenshotFilename,
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
}
