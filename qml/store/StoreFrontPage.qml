import QtQuick 2.4

Item {
    width: 400
    height: 400

    Rectangle {
        id: rectangle1
        x: 22
        y: 18
        width: 529
        height: 315
        color: "#ff00ff"
    }

    Text {
        id: text1
        x: 567
        y: 18
        text: "Something something"
        font.pixelSize: 32
    }

    Text {
        id: text2
        x: 567
        y: 62
        width: 395
        height: 271
        text: qsTr("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec eu nisl auctor, ultrices dui non, pulvinar nibh. Phasellus feugiat sollicitudin ullamcorper. Donec commodo dolor vitae mi malesuada, eu fringilla nisi laoreet. Nunc id augue et diam bibendum ultrices. Aliquam erat volutpat. Cras est odio, feugiat sed orci sed, rhoncus molestie nulla. Aenean consectetur eget odio eget iaculis.\n\nPraesent hendrerit vitae dui sit amet tincidunt. Vivamus rutrum interdum auctor. Aliquam erat volutpat. Donec lacus metus, sagittis posuere suscipit et, iaculis sed sem. Donec sit amet interdum massa. Nullam dignissim eleifend cursus. Pellentesque eget dolor aliquet, sagittis magna sit amet, mollis dui. Morbi sit amet rhoncus elit. Maecenas pretium suscipit vestibulum. ")
        wrapMode: Text.WrapAtWordBoundaryOrAnywhere
        font.pixelSize: 12
    }

    Text {
        id: text3
        x: 22
        y: 345
        text: "Simulations"
        font.pixelSize: 32
    }

    Row {
        id: row1
        x: 22
        y: 389
        width: 940
        height: 213

        Repeater {
            model: ListModel {
                ListElement {
                    name: "Woop woop"
                }
                ListElement {
                    name: "Doop asfaf"
                }
                ListElement {
                    name: "Lol woop"
                }
            }
            delegate: StoreItem {
                titleText.text: name
            }
        }
    }
}
