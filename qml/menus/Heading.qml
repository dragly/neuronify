import QtQuick 2.0
import "../style"

Text {
    id: welcomeText
    color: Style.heading.color
    font: Style.heading.font
    renderType: Qt.platform.os === "linux" ? Text.NativeRendering : Text.QtRendering
}

