import QtQuick 2.0

ListModel {
    ListElement {
        name: "Visual input"
        description: "Generates spikes based on visual input"
        source: "../sensors/Retina.qml"
        imageSource: "qrc:/images/sensors/eye.png"
    }

    ListElement {
        name: "Touch activator"
        description: "Generates spikes in neurons it is connected to based on touch"
        source: "../sensors/TouchSensor.qml"
        imageSource: "qrc:/images/sensors/touch_sensor.png"
    }
}
