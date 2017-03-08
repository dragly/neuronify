import QtQuick 2.4
import QtQuick.Controls 2.0

import "qrc:/qml/backend"
import "qrc:/qml/firebase"
import "qrc:/qml/style"

Item {
    id: root

    signal clicked(string objectId)

    width: 400
    height: 400

    Component.onCompleted: {
        Parse.get("Simulation", function(response) {
            for(var i in response.results) {
                var simulation = response.results[i]
                listModel.append(response.results[i])
            }
        })
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
                        description: model.description ? model.description : ""
                        imageUrl: model.image ? model.image.url : ""
                        onClicked: {
                            root.clicked(model.objectId)
                        }
                    }
                }
            }
        }
    }
}
