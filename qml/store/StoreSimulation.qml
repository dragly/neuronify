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
    signal deleted()

    property string name: objectData ? objectData.name : ""
    property string description: objectData ? objectData.description : ""
    property string creator: "The Creator Company Inc."
    property var objectData

    property url imageUrl
    property real price: 0.0

    Material.theme: Material.Light

    onObjectDataChanged: {
        Firebase.cachedDownload(
                    objectData.screenshot,
                    function (localFileName) {
                        imageUrl = localFileName
                    })
    }

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

                Rectangle {
                    id: imageRectangle
                    anchors {
                        top: parent.top
                        bottom: parent.bottom
                        left: parent.left
                    }

                    width: height

                    Image {
                        id: image

                        anchors.fill: parent

                        antialiasing: true
                        smooth: true

                        source: root.imageUrl
                        fillMode: Image.PreserveAspectCrop
                    }
                }

                Column {
                    anchors {
                        top: parent.top
                        left: imageRectangle.right
                        bottom: parent.bottom
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

                Button {
                    visible: objectData && objectData.uid === Firebase.userId ? true : false
                    text: "Delete"
                    onClicked: {
                        for (var i in objectData) {
                            console.log(i)
                        }

                        Firebase.remove(
                                    "simulations/" + objectData._key + ".json",
                                    function (response) {
                                        root.deleted()
                                    })
                    }
                }

                Button {
                    id: runButton

                    text: "Run"
                    onClicked: {
                        Firebase.cachedDownload(objectData.simulation, function(localFilename) {
                            FileIO.read(localFilename, function(data) {
                                var simulation = {
                                    name: objectData.name,
                                    description: objectData.description,
                                    data: data,
                                }
                                root.runClicked(simulation)
                            })
                        })
                    }
                }
            }
        }

    }
}
