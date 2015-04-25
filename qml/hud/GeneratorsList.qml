import QtQuick 2.0

CreationList {
    id: itemRow

    CreationItem {
        name: "Current clamp"
        description: "Generates constant current clamped to neurons."
        source: "qrc:/qml/generators/CurrentClamp.qml"
        imageSource: "qrc:/images/creators/generators/current_clamp.png"
    }
}
