import QtQuick 2.0

ListModel {
    ListElement {
        name: "Visual input"
        description: "Generates spikes based on visual input"
        source: "qrc:/qml/sensors/Retina.qml"
        imageSource: "qrc:/images/sensors/eye.png"
    }

    ListElement {
        name: "Touch activator"
        description: "Generates spikes in neurons it is connected to based on touch"
        source: "qrc:/qml/sensors/TouchSensor.qml"
        imageSource: "qrc:/images/sensors/touch_sensor.png"
    }
}
