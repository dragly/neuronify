import QtQuick 2.0
import Neuronify 1.0
import QtQuick.Controls 1.4

import "../../controls"

DogKernelEngine{
    id: dogEngine
    resolutionHeight: 20
    resolutionWidth: 20

    property Component controls: Component{
        Column{
            spacing: 10
            Text{
                text: "Type: " + (!switchRoot.checked ? " On-center" : " Off-center")
            }
            Switch{
                id: switchRoot
                checked: dogEngine.isOffCenter
            }

            Binding {
                target: dogEngine
                property: "isOffCenter"
                value: switchRoot.checked
            }

        }

    }


    savedProperties: PropertyGroup {
        property alias isOffCenter: dogEngine.isOffCenter
    }

}



