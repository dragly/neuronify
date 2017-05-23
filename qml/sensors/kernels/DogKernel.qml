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
    }

}



