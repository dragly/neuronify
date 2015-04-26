import QtQuick 2.0

CreationList {
    id: itemRow

    CreationItem {
        name: "Retina input"
        description: "Generates spikes based on visual input."
        source: "../sensors/Retina.qml"
        imageSource: "qrc:/images/sensors/eye.png"
    }

    CreationItem {
        name: "Touch sensor"
        description: "Gives a current output based on touch."
        source: "../sensors/TouchSensor.qml"
        imageSource: "qrc:/images/sensors/touch_sensor.png"
    }
}
