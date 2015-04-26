import QtQuick 2.0

CreationList {
    id: itemRow

    CreationItem {
        name: "Voltmeter"
        description: "Measures the voltage of neurons."
        source: "qrc:/qml/meters/Voltmeter.qml"
        imageSource: "qrc:/images/meters/voltmeter.png"
    }
}
