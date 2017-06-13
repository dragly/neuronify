import QtQuick 2.7
import QtQuick.Controls 2.0
import QtQuick.Controls.Material 2.0
import QtQuick.Layouts 1.0
import QtGraphicalEffects 1.0

import Neuronify 1.0

import "qrc:/qml/backend"

Item {
    id: root

    signal runClicked(var simulation)

    property DownloadManager downloadManager

    property string name: objectData.name
    property string description: objectData.description
    property string creator: "The Creator Company Inc."
    property var objectData: {
        return {
            "createdAt":"2017-03-09T08:48:35.368Z",
            "description":"hello",
            "name":"test",
            "objectId":"m4JkohPzJ8",
            "screenshot":{
               "__type":"File",
               "name":"53a3dc41efa39bf5162678b7f581f622_screenshot.png",
               "url":"https://parsefiles.back4app.com/JffGes20AXUtdN9B6E1RkkHaS7DOxVmxJFSJgLoN/53a3dc41efa39bf5162678b7f581f622_screenshot.png"
            },
            "simulation":{
               "__type":"File",
               "name":"b4d41543780cda16f4f06b2aa6334f12_simulation.nfy",
               "url":"https://parsefiles.back4app.com/JffGes20AXUtdN9B6E1RkkHaS7DOxVmxJFSJgLoN/b4d41543780cda16f4f06b2aa6334f12_simulation.nfy"
            },
            "updatedAt":"2017-03-09T08:48:35.368Z"
        }
    }

    property url imageUrl: objectData.screenshot.url
    property real price: 0.0
    readonly property url targetLocation: StandardPaths.writableLocation(StandardPaths.AppDataLocation, "community/" + objectData.objectId)
    readonly property url simulationPath: targetLocation + "/simulation.nfy"
    readonly property bool downloaded: FileIO.exists(simulationPath) // TODO replace with database

    Material.theme: Material.Light

    onObjectDataChanged: console.log(imageUrl, objectData.screenshot.url)

    width: 1200
    height: 600
    
    FontMetrics {
        id: defaultMetric
    }

    Flickable {
        anchors.fill: parent

        contentHeight: contents.height
        contentWidth: contents.width

        flickableDirection: Flickable.VerticalFlick
        ScrollBar.vertical: ScrollBar {}


        Rectangle {
            id: background

            anchors {
                fill: contents
            }

            color: Material.background
        }

        StoreShadow {
            anchors.fill: background
            source: background
        }

        Column {
            id: contents
            anchors {
                top: parent.top
                horizontalCenter: parent.horizontalCenter
            }

            width: 640

            spacing: 24
            padding: 24

            Item {
                id: headerContents
                anchors {
                    left: parent.left
                    right: parent.right
                    margins: 24
                }

                height: 256

                Image {
                    id: image
                    anchors {
                        top: parent.top
                        bottom: parent.bottom
                        left: parent.left
                    }

                    width: height

                    source: root.imageUrl
                    fillMode: Image.PreserveAspectCrop
                }

                Column {
                    anchors {
                        top: parent.top
                        bottom: parent.bottom
                        left: image.right
                        leftMargin: 24
                        right: parent.right
                    }

                    clip: true
                    spacing: 16

                    Label {
                        id: nameLabel

                        anchors {
                            left: parent.left
                            right: parent.right
                        }

                        font.pixelSize: defaultMetric.font.pixelSize * 2.4
                        text: root.name
                        wrapMode: Label.WrapAtWordBoundaryOrAnywhere
                    }

//                    Label {
//                        id: creatorLabel
//                        text: root.creator
//                        font.weight: defaultMetric.font.weight * 1.4
//                    }

//                    Label {
//                        id: priceLabel
//                        text: root.price > 0 ? "NOK" + root.price.toFixed(2) : "FREE"
//                    }
                }
            }

            Label {
                id: descriptionLabel
                anchors {
                    left: parent.left
                    right: parent.right
                    margins: 24
                }

                wrapMode: Label.WrapAtWordBoundaryOrAnywhere

                text: root.description
            }

            Row {
                anchors {
                    right: parent.right
                    margins: 24
                }
                spacing: 8

                // TODO add back install button when database is ready

//                Button {
//                    id: installButton
//                    text: downloaded ? "Reinstall" : "Install"
//                    onClicked: {
//                        FileIO.write(targetLocation + "/info.json", JSON.stringify(objectData, null, 4))
//                        if(objectData.simulation) {
//                            downloadManager.download(
//                                        objectData.simulation.url,
//                                        targetLocation + "/simulation.nfy")
//                        }
//                        if(objectData.screenshot) {
//                            downloadManager.download(
//                                        objectData.screenshot.url,
//                                        targetLocation + "/screenshot.png")
//                        }
//                    }
//                }

                Button {
                    id: runButton

                    text: "Run"
                    onClicked: {
                        Parse.download(objectData.simulation.url, function(data) {
                            var simulation = {
                                name: objectData.name,
                                description: objectData.description,
                                data: data,
                            }
                            root.runClicked(simulation)
                        })
                    }
                }
            }
        }

    }
}
