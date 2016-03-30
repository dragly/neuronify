import QtQuick 2.0
import Neuronify 1.0
import QtQuick.Controls 1.4
import QtQuick.Controls.Styles 1.4
import "../../controls"
import "qrc:/qml/style"

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

                Gradient {
                    id: grad
                    GradientStop { position: 0.0; color: Style.color.foreground }
                    GradientStop { position: 1.0; color: Style.color.border }
                }

                style: SwitchStyle {
                    groove: Rectangle {
                            implicitWidth: 100
                            implicitHeight: 30
                            radius: height/2
                            color: Style.color.background
                            border.color: "#9ecae1"
                            border.width: Style.border.width
                    }

                    handle: Rectangle {
                        implicitWidth: 100 * 0.5
                        implicitHeight: 30
                        radius: height/2
                        color: "#9ecae1"
                        border.color: "#9ecae1"
                        border.width: Style.border.width
                        gradient: grad
                    }
                }



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



