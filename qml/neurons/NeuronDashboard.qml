import QtQuick 2.0
import QtCharts 2.1
import QtQuick.Controls 2.1
import QtQuick.Controls.Material 2.1
import QtQuick.Layouts 1.1

import Neuronify 1.0

Rectangle {
    id: root
    color: "#def"

    property real timeFactor: 1e3
    property real voltageFactor: 1e3

    width: 640
    height: 480

    TabBar {
        id: tabBar
        anchors {
            left: parent.left
            right: parent.right
            top: parent.top
        }

        TabButton {
            text: "Potentials"
        }
        TabButton {
            text: "Membrane"
        }
        TabButton {
            text: "Synapse"
        }
    }

    RowLayout {
        anchors {
            top: tabBar.bottom
            bottom: parent.bottom
            left: parent.left
            right: parent.right
        }

        ColumnLayout {
            Layout.fillHeight: true
            Layout.fillWidth: true
            Label {
                anchors {
                    horizontalCenter: thresholdSlider.horizontalCenter
                }
                text: "Threshold"
            }
            ChartSlider {
                id: thresholdSlider
                Layout.fillHeight: true

                Binding {
                    target: thresholdSlider
                    property: "chart"
                    value: chart_
                }
            }
        }

        ColumnLayout {
            Layout.fillHeight: true
            Layout.fillWidth: true
            Label {
                text: "Reset"
            }

            ChartSlider {
                id: resetSlider

                Layout.fillHeight: true

                Binding {
                    target: resetSlider
                    property: "chart"
                    value: chart_
                }
            }
        }

        ChartView {
            id: chart_

            Layout.fillHeight: true
            Layout.fillWidth: true

            antialiasing: true
            smooth: true
            backgroundColor: Material.primary
            legend.visible: false
            z: -1

            LineSeries {
               id: series
               axisX: axisX
               axisY: axisY
               color: "white"
               width: 3
            }

            LineSeries {
               id: fireSeries
               axisX: axisX
               axisY: axisY
               color: "white"
               width: 3
            }

            ValueAxis {
                id: axisX
                min:  0
                max:  100
                gridVisible: false
                color: "white"
                lineVisible: false
                labelsColor: "white"
            }

            ValueAxis {
                id: axisY
                min: -100
                max: 30
                gridVisible: false
                lineVisible: false
                labelsColor: "white"
            }
        }

        ColumnLayout {
            Layout.fillHeight: true
            Layout.fillWidth: true

            Label {
                anchors {
                    horizontalCenter: restingSlider.horizontalCenter
                }
                text: "Resting"
            }
            ChartSlider {
                id: restingSlider
                Layout.fillHeight: true

                Binding {
                    target: restingSlider
                    property: "chart"
                    value: chart_
                }
            }
        }

    }



    function refresh() {
        console.log("REFRESH!")
        series.clear()
        fireSeries.clear()

        var t = 0
        var dt = 1e-3
        engine.resetDynamics()
        var current = 0.1e-9
        var didFireSometime = false
        for(var i = 0; i < 200; i++) {
            if(!didFireSometime) {
                current = - (1.0 / leak.resistance) * (engine.voltage - (engine.threshold + (engine.threshold - engine.restingPotential) + 10e-3))
                engine.receiveCurrent(current, null)
            }
            engine.step(dt, true)
//            console.log(engine.voltage * voltageFactor)
            series.append(t*timeFactor, engine.voltage*voltageFactor)

            if(engine.hasFire) {
                fireSeries.append((t - dt) * timeFactor - 1e-1, 1000e-3 * voltageFactor)
                fireSeries.append((t - dt) * timeFactor, engine.voltage * voltageFactor)
                fireSeries.append((t - dt) * timeFactor + 1e-1, 1000e-3 * voltageFactor)
                didFireSometime = true
            }

            engine.hasFire = false

//            console.log(t, engine.voltage)
            t += dt
        }
    }

    NeuronEngine {
        id: engine

        property bool hasFire: false

//        capacitance: capacitanceSlider.value * 1e-9
        threshold: thresholdSlider.cutValue * 1e-3
        restingPotential: restingSlider.cutValue * 1e-3
        initialPotential: resetSlider.cutValue * 1e-3

        Component.onCompleted: {
            refresh()
        }

        onRestingPotentialChanged: {
            refresh()
        }

        onInitialPotentialChanged: {
            refresh()
        }

        onThresholdChanged: {
            refresh()
        }

        onCapacitanceChanged: {
            refresh()
        }

        onFired: {
            hasFire = true
//            console.log("FIRED")
        }


        LeakCurrent {
            id: leak

//            resistance: resistanceSlider.value * 1e6
            onResistanceChanged: {
                refresh()
            }
        }
    }

    Column {
//        Slider {
//            id: capacitanceSlider
//            from: 0.2
//            to: 2
//            value: 1
//        }

//        Slider {
//            id: resistanceSlider
//            from: 2
//            to: 50
//            value: 25
//        }

//        Slider {
//            id: restingSlider
//            from: -100
//            to: 30
//            value: -70
//        }
    }
}
