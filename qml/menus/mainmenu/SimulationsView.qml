import QtQuick 2.6
import QtQuick.Controls 1.4

import "qrc:/qml/style"
import "qrc:/qml/tools"

MainMenuPage {
    id: simulationsViewRoot

    signal simulationClicked(var simulation)
    signal backClicked

    title: "Examples"
    clip: true

    ScrollView {
        anchors.fill: parent
        Flickable {
            anchors {
                fill: parent
                margins: Style.baseMargin
            }

            contentWidth: width
            contentHeight: categories.height
            flickableDirection: Flickable.VerticalFlick
            Column {
                id: categories
                anchors{
                    left: parent.left
                    right: parent.right
                }
                spacing: Style.baseMargin
                Repeater {
                    model: [
                        {
                            name: "Tutorial",
                            simulations: [
                                "qrc:/simulations/tutorial/tutorial_1_intro",
                                "qrc:/simulations/tutorial/tutorial_2_circuits",
                                "qrc:/simulations/tutorial/tutorial_3_creation",
                            ]
                        },
                        {
                            name: "Neuronify Items",
                            simulations: [
                                "qrc:/simulations/items/neurons/leaky",
                                "qrc:/simulations/items/neurons/inhibitory",
    //                            "qrc:/simulations/items/neurons/burst",
                                "qrc:/simulations/items/neurons/adaptation",
                                "qrc:/simulations/items/visualInput",
                                "qrc:/simulations/items/generators",
                                "qrc:/simulations/items/frPlot",

                            ]
                        },
                        {
                            name: "Miscellaneous",
                            simulations: [
                                "qrc:/simulations/mix/lateral_inhibition",
                                "qrc:/simulations/mix/recurrent_inhibition",
                                "qrc:/simulations/mix/reciprocal_inhibition",
                                "qrc:/simulations/mix/disinhibition",
                                "qrc:/simulations/mix/lateral_inhibition_1",
                                "qrc:/simulations/mix/lateral_inhibition_2",
                                "qrc:/simulations/mix/input_summation",
                                "qrc:/simulations/mix/rythm_transformation",
                                "qrc:/simulations/mix/prolonged_activity",

                            ]
                        },
                        {
                            name: "Textbook Examples",
                            simulations: [
                                "qrc:/simulations/sterratt/if_response",
                                "qrc:/simulations/sterratt/refractory_period",
                            ]
                        },
                    ]
                    Column {
                        spacing: Style.baseMargin
                        Text {
                            text: modelData.name
                            font: Style.button.font
                            color: Style.button.color
                        }
                        ScrollView{
                            width: categories.width
                            height: simulationsRow.height + Style.baseMargin
                            Flickable {
                                anchors.fill: parent
                                contentHeight: height
                                contentWidth: simulationsRow.width
                                flickableDirection: Flickable.HorizontalFlick
                                Row {
                                    id: simulationsRow
                                    spacing: Style.baseMargin * 0.5
                                    Repeater {
                                        model: modelData.simulations
                                        Item {
                                            width: simulationColumn.width
                                            height: simulationColumn.height
                                            SimulationLoader {
                                                id: loader
                                                folder: modelData
                                            }
                                            Column {
                                                id: simulationColumn
                                                Image {
                                                    id: simulationImage
                                                    property int simulationsCount: simulationsViewRoot.width / (Style.touchableSize * 2.5 + Style.baseMargin)

                                                    width: (simulationsViewRoot.width - Style.baseMargin) / (simulationsCount + 0.3) - simulationsRow.spacing
                                                    height: width

                                                    source: loader.item ? loader.item.screenshotSource : ""
                                                    smooth: true
                                                    antialiasing: true
                                                    fillMode: Image.PreserveAspectCrop
                                                    asynchronous: true

                                                    Rectangle {
                                                        color: "transparent"
                                                        width: parent.width
                                                        height: parent.height
                                                        border.color: Style.border.color
                                                        border.width: Style.border.width
                                                    }
                                                }
                                                Text {
                                                    id: title
                                                    font: Style.text.font
                                                    color: Style.text.color
                                                    text: loader.item ? loader.item.name : "N/A"
                                                    width: simulationImage.width
                                                    wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                                                    horizontalAlignment: Text.AlignHCenter

                                                }
                                            }
                                            MouseArea {
                                                anchors.fill: parent
                                                onClicked: {
                                                    simulationClicked(loader.item.stateSource)
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

