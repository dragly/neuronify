import QtQuick 2.7
import QtQuick.Controls 2.0
import QtQuick.Controls.Material 2.0
import QtQuick.Layouts 1.0
import QtGraphicalEffects 1.0

Item {
    id: root
    
    property string objectId: "oSIhtssGMj"
    property string name: "Some name of a simulation without a name yet and this has a long name"
    property string description: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aenean sit amet tempor dui. Nam maximus tempus tortor a porttitor. Curabitur faucibus convallis dui, at dictum diam euismod eu. Sed sit amet eleifend tellus. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Morbi a erat nec augue egestas sodales. Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Sed eget ex in felis lacinia sodales. Praesent semper sagittis eros non sodales. Nam eu mollis orci. Sed vehicula efficitur felis, nec ornare enim ullamcorper quis."
    property string creator: "The Creator Company Inc."
    property url imageUrl
    property real price: 0.0

    width: 1200
    height: 600

    Component.onCompleted: {
        reload()
    }

    onObjectIdChanged: {
        reload()
    }

    function reload() {
        var status
        var wasLoading
        var req = new XMLHttpRequest;
        req.open("GET", "http://neuronify.ovilab.net:1337/parse/classes/Simulation/" + objectId + "/?include=creator");
        req.setRequestHeader("X-Parse-Application-Id", "neuronify");
        req.onreadystatechange = function() {
            status = req.readyState;
            if (status === XMLHttpRequest.DONE) {
                console.log("Response", req.responseText)
                var object = JSON.parse(req.responseText);
                if (object.error)
                    console.log("Error fetching data: " + object.error)
                else {
                    console.log("Got result", object)
                    root.name = object.name
                    root.description = object.description
                    root.imageUrl = object.image.url
                    root.creator = object.creator.name
                    root.price = object.price
                }
                if (wasLoading == true) {
                    console.log("Is loaded")
                }
            }
            wasLoading = (status === XMLHttpRequest.LOADING);
        }
        req.send();
    }
    
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

            width: 800

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

                    Label {
                        id: creatorLabel
                        text: root.creator
                        font.weight: defaultMetric.font.weight * 1.4
                    }

                    Label {
                        id: priceLabel
                        text: root.price > 0 ? "NOK" + root.price.toFixed(2) : "FREE"
                    }
                }
            }

            Button {
                id: installButton
                anchors {
                    right: parent.right
                    margins: 24
                }

                text: root.price > 0 ? "BUY NOK " + root.price.toFixed(2) : "Install"
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
        }

    }
}
