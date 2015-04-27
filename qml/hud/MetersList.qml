import QtQuick 2.0

CreationList {
    id: itemRow

    CreationItem {
        name: "Voltmeter"
        description: "Measures the voltage of neurons."
        source: "qrc:/qml/meters/Voltmeter.qml"
        imageSource: "qrc:/images/meters/voltmeter.png"
    }

    CreationItem {
        name: "Speaker"
        description: "Plays a sound when a connected object fires."
        source: "qrc:/qml/meters/Speaker.qml"
        imageSource: "qrc:/images/meters/speaker.png"
    }
}
