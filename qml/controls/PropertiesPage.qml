import QtQuick 2.6
import QtQuick.Controls 1.4

import "qrc:/qml/style"

Column {
    property string title
    property StackView stackView: Stack.view
    spacing: Style.spacing
}
