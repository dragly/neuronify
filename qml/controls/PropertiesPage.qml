import QtQuick 2.6
import QtQuick.Controls 2.1

import "qrc:/qml/style"

Item {
    property string title
    property StackView stackView: null
    default property alias _data: column.data

    width: 320
    height: 320

    Column {
        id: column
        anchors {
            fill: parent
            margins: 24
        }

        spacing: Style.spacing
    }
}
