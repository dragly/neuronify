import QtQuick 2.0

CreationList {
    id: itemRow
    CreationItem {
        name: "Touch sensor"
        description: "Gives a current output based on touch."
        source: "../sensors/TouchSensor.qml"
        imageSource: "qrc:/images/sensors/touch_sensor.png"
    }
}
