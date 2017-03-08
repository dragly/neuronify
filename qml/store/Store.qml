import QtQuick 2.7
import QtQuick.Controls 2.0
import QtQuick.Layouts 1.0
import QtGraphicalEffects 1.0
//import QtPurchasing 1.0 as QP

import "qrc:/qml/backend"
import "qrc:/qml/style"

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

    Item {
        id: rectangle2
//        color: "#eee"
//        padding: 0
        anchors.right: parent.right
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        anchors.top: parent.top

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

        Column {
            anchors {
                top: rectangle3.bottom
                left: parent.left
                right: rectangle3.right
                margins: 8
            }

            Button {
                width: 120
                text: "Upload items"
            }

            Button {
                width: 120
                text: "Sign up"
                onClicked: {
                    Parse.post("_User", '{"username":"cooldude6","password":"p_n7!-e8","phone":"415-392-0202"}')
                }
            }

            Button {
                width: 120
                text: "Log in"
                onClicked: {
                    Parse.login("cooldude6", "p_n7!-e8")
                }
            }
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
