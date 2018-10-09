import QtQuick 2.0
import Neuronify 1.0
import QtQuick.Controls 2.1
import QtQuick.Controls.Styles 1.4
import "../../controls"
import "qrc:/qml/style"

DogKernelEngine{
    id: dogEngine
    resolutionHeight: 20
    resolutionWidth: 20

    property Component controls: Component{
        Column{
            BoundSlider {
                target: dogEngine
                property: "xOffset"
                minimumValue: -0.5
                maximumValue: 0.5
                unitScale: 0.1
                stepSize: 0.1
                precision: 1
                text: "x"
                unit: ""
            }

            BoundSlider {
                target: dogEngine
                property: "yOffset"
                minimumValue: -0.5
                maximumValue: 0.5
                unitScale: 0.1
                stepSize: 0.1
                precision: 1
                text: "y"
                unit: ""
            }

            SwitchControl{
                id: switchControl
                target: dogEngine
                property: "isOffCenter"
                checkedText: "Off center"
                uncheckedText: "On center"
            }


        }

    }


    savedProperties: PropertyGroup {
        property alias isOffCenter: dogEngine.isOffCenter
        property alias xOffset: dogEngine.xOffset
        property alias yOffset: dogEngine.yOffset
    }

}



