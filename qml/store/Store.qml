import QtQuick 2.7
import QtQuick.Controls 2.0
import QtQuick.Layouts 1.0

Item {
    width: 1280
    height: 900
    Rectangle {
        id: rectangle1
        height: 65
        color: "#ffffff"
        anchors.right: parent.right
        anchors.rightMargin: 0
        anchors.left: parent.left
        anchors.leftMargin: 0
        anchors.top: parent.top
        anchors.topMargin: 0

        Label {
            id: text1
            y: 26
            text: qsTr("Neuronify Store")
            font.pixelSize: 24
            anchors.left: parent.left
            anchors.leftMargin: 16
            anchors.verticalCenter: parent.verticalCenter
        }

        TextField {
            id: textField1
            x: 851
            width: 373
            text: qsTr("")
            anchors.bottom: parent.bottom
            anchors.bottomMargin: 16
            anchors.top: parent.top
            anchors.topMargin: 16
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
        color: "#e4e7ed"
        anchors.right: parent.right
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        anchors.top: rectangle1.bottom

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
                            colorCode: "#D13F32"
                        }

                        ListElement {
                            name: "Neurons"
                            colorCode: "#1D7872"
                        }

                        ListElement {
                            name: "Items"
                            colorCode: "#71B095"
                        }

                        ListElement {
                            name: "Plugins"
                            colorCode: "#1A212C"
                        }
                    }
                    delegate: Rectangle {
                        id: rectangle4
                        height: 64
                        color: colorCode
                        anchors.right: parent.right
                        anchors.left: parent.left

                        Label {
                            id: text2
                            color: "#ffffff"
                            text: name
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: parent.left
                            anchors.leftMargin: 16
                            font.pixelSize: 16
                        }
                    }
                }
            }
        }

        Item {
            id: contentContainer
            anchors.margins: 16
            anchors.left: rectangle3.right
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.top: parent.top

            StoreItem {}
        }
    }
}
