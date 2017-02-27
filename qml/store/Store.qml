import QtQuick 2.7
import QtQuick.Controls 2.1
import QtQuick.Layouts 1.0
import QtGraphicalEffects 1.0
//import QtPurchasing 1.0 as QP

Item {
    width: 1280
    height: 900

    property var colors: [
        "#05668d",
        "#028090",
        "#00a896",
        "#02c39a",
        "#f0f3bd",
    ]

//    QP.Store {
//        QP.Product {
//            identifier: "testProduct"
//            type: QP.Product.Unlockable
//            onPurchaseSucceeded: {
//                console.log("Purchase OK")
//            }
//        }
//    }

    ToolBar {
        id: headerRectangle
        height: 65
//        color: "#ffffff"
        anchors {
            right: parent.right
            left: parent.left
            top: parent.top
        }
        z: 9999

        Label {
            id: text1
            anchors {
                left: parent.left
                leftMargin: 16
                verticalCenter: parent.verticalCenter
            }
            text: qsTr("Neuronify Store")
            font.pixelSize: 24
        }

        Button {
            id: backButton
            anchors {
                left: text1.right
                verticalCenter: parent.verticalCenter
                margins: 16
            }

            text: "Back"

            onClicked: stackView.pop()
        }

        TextField {
            id: textField1
            x: 851
            width: 373
            text: qsTr("")
            anchors.bottom: parent.bottom
            anchors.bottomMargin: 4
            anchors.top: parent.top
            anchors.topMargin: 4
            anchors.right: item2.left
            anchors.rightMargin: 0
            placeholderText: qsTr("Search")
        }

        Item {
            id: item2
            width: height
            anchors.bottom: parent.bottom
            anchors.bottomMargin: 16
            anchors.top: parent.top
            anchors.topMargin: 16
            anchors.right: parent.right
            anchors.rightMargin: 8

            MouseArea {
                id: mouseArea1
                anchors.fill: parent
            }

            Image {
                id: image1
                anchors.rightMargin: 0
                anchors.fill: parent
                source: "qrc:/images/store/ic_search_black_48dp.png"
            }
        }
    }

    Rectangle {
        id: rectangle2
        color: "#eee"
//        padding: 0
        anchors.right: parent.right
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        anchors.top: headerRectangle.bottom

        Rectangle {
            id: rectangle3
            width: 200
            height: column1.height
            color: "#ffffff"
            anchors.left: parent.left
            anchors.leftMargin: 0
            anchors.top: parent.top
            anchors.topMargin: 0

            Column {
                id: column1
                y: 0
                anchors.right: parent.right
                anchors.rightMargin: 0
                anchors.left: parent.left
                anchors.leftMargin: 0

                Repeater {
                    model: ListModel {
                        ListElement {
                            name: "Simulations"
                        }

                        ListElement {
                            name: "Neurons"
                        }

                        ListElement {
                            name: "Items"
                        }

                        ListElement {
                            name: "Plugins"
                        }
                    }
                    delegate: ItemDelegate {
                        id: rectangle4
                        anchors {
                            left: parent.left
                            right: parent.right
                        }

                        height: 56

                        Rectangle {
                            id: colorRectangle
                            anchors {
                                top: parent.top
                                bottom: parent.bottom
                                left: parent.left
                            }

                            width: 48

                            color: colors[index]
                        }


                        Label {
                            id: text2
                            anchors {
                                verticalCenter: parent.verticalCenter
                                left: colorRectangle.right
                                leftMargin: 16
                            }
                            text: name
                        }
                    }
                }
            }
        }

        StoreShadow {
            source: rectangle3
            anchors.fill: rectangle3
        }

        Button {
            anchors {
                top: rectangle3.bottom
                left: parent.left
                right: rectangle3.right
                margins: 8
            }

            height: 56

            text: "Upload items"
        }

        Item {
            id: contentContainer
            anchors {
                margins: 16
                left: rectangle3.right
                right: parent.right
                bottom: parent.bottom
                top: parent.top
            }

            clip: true

            StackView {
                id: stackView
                anchors.fill: parent
                initialItem: mainView
            }
        }

        Component {
            id: mainView
            StoreFrontPage {
                width: parent.width
                height: parent.height

                onClicked: {
                    var item = stackView.push(simulationView)
                    item.objectId = objectId
                }
            }
        }

        Component {
            id: simulationView
            StoreSimulation {
                id: root
                width: parent ? parent.width : 400
                height: parent ? parent.height : 400
            }
        }
    }
}
