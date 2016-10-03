import QtQuick 2.4
import QtQuick.Controls 2.0

Item {
    id: root

    signal clicked(string objectId)

    width: 400
    height: 400

    Component.onCompleted: {
        var status
        var wasLoading
        var req = new XMLHttpRequest;
        req.open("GET", "http://neuronify.ovilab.net:1337/parse/classes/Simulation");
        req.setRequestHeader("X-Parse-Application-Id", "neuronify");
        req.onreadystatechange = function() {
            status = req.readyState;
            if (status === XMLHttpRequest.DONE) {
                var objectArray = JSON.parse(req.responseText);
                if (objectArray.errors !== undefined)
                    console.log("Error fetching tweets: " + objectArray.errors[0].message)
                else {
                    for(var i in objectArray.results) {
                        var simulation = objectArray.results[i]
                        listModel.append(objectArray.results[i])
                    }
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
        contentWidth: column.width
        contentHeight: column.height
        clip: true

        flickableDirection: Flickable.VerticalFlick

        ScrollBar.vertical: ScrollBar {}

        Column {
            id: column
            anchors {
                left: parent.left
                top: parent.top
                margins: 16
            }

            spacing: 16

            Text {
                id: text3
                text: "Simulations"
                font.pixelSize: defaultMetric.font.pixelSize * 1.6
            }

            Row {
                id: row1
                spacing: 16

                Repeater {
                    model: ListModel { id: listModel }
                    delegate: StoreItem {
                        width: 160
                        height: 256
                        name: model.name
                        description: model.description
                        price: model.price > 0 ? "NOK " + model.price.toFixed(2) : "Free"
                        imageUrl: model.image.url
                        onClicked: {
                            root.clicked(model.objectId)
                        }
                    }
                }
            }
        }
    }
}
