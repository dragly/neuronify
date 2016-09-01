import QtQuick 2.4

Item {
    id: item1
    width: 320
    height: 480
    property alias titleText: titleText
    property alias descriptionText: descriptionText
    property alias priceText: priceText

    Rectangle {
        id: rectangle1
        color: "#ffffff"
        anchors.bottom: titleText.top
        anchors.bottomMargin: 8
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.left: parent.left

        Image {
            id: image1
            fillMode: Image.PreserveAspectCrop
            anchors.fill: parent
            source: "qrc:/images/store/ic_search_black_48dp.png"
        }
    }

    Text {
        id: titleText
        y: 361
        height: 34
        text: qsTr("Some items")
        anchors.bottom: descriptionText.top
        anchors.bottomMargin: 8
        anchors.right: parent.right
        anchors.rightMargin: 8
        anchors.left: parent.left
        anchors.leftMargin: 8
        font.pixelSize: 24
    }

    Text {
        id: descriptionText
        y: 396
        text: qsTr("Neural networks")
        anchors.left: parent.left
        anchors.leftMargin: 8
        anchors.bottom: priceText.top
        anchors.bottomMargin: 8
        font.pixelSize: 12
    }

    Text {
        id: priceText
        x: 286
        y: 458
        text: qsTr("Free")
        anchors.right: parent.right
        anchors.rightMargin: 8
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 8
        font.pixelSize: 12
    }
}
