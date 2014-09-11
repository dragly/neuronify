import QtQuick 2.0
import QtQuick.Controls 1.2

Rectangle {
    width: 400
    height: 300

    Compartment {
        id: compartment
        forceTargetVoltage: targetVoltageCheckbox.checked
        targetVoltage: targetVoltageSlider.value
    }

    Column {
        anchors {
            right: parent.right
            top: parent.top
        }
        Button {
            id: polarizeButton

            text: "Polarize!"
            onClicked: {
                compartment.voltage += 90
            }
        }

        Button {
            id: depolarizeButton

            text: "Depolarize!"
            onClicked: {
                compartment.voltage -= 20
            }
        }

        Button {
            id: resetButton

            text: "Reset!"
            onClicked: {
                compartment.voltage = 0
                compartment.sodiumActivation = 0
                compartment.potassiumActivation = 0
                compartment.sodiumInactivation = 0
            }
        }

        CheckBox {
            id: targetVoltageCheckbox
            text: "Lock to target voltage"
        }

        Text {
            text: "Target voltage: " + targetVoltageSlider.value.toFixed(1) + " mV"
        }

        Slider {
            id: targetVoltageSlider
            minimumValue: -120
            maximumValue: 80.0
        }
    }

    Canvas {
        id: plot
        property color strokeStyle:  Qt.darker(fillStyle, 1.4)
        property color fillStyle: "#b40000" // red
        property real alpha: 1.0
        property var data: new Array(1000)
        antialiasing: true

        onScaleChanged: requestPaint()
        onDataChanged: requestPaint()
        anchors {
            bottom: parent.bottom
            left: parent.left
            right: parent.right
        }
        height: parent.height / 2.0

        function addPoint(point) {
            var newData = data
            newData.shift()
            newData.push(point)
            data = newData
        }

        onPaint: {
            var ctx = getContext("2d")
            ctx.save();
            ctx.clearRect(0, 0, plot.width, plot.height);
            ctx.globalAlpha = plot.alpha;
            ctx.strokeStyle = plot.strokeStyle;
            ctx.fillStyle = plot.fillStyle;
            ctx.lineWidth = plot.lineWidth;
            ctx.beginPath();
            ctx.moveTo(0,100 - data[0]);
            var x = 0;
            for(var i in data) {
                var y = data[i];
                ctx.lineTo(x,100 - y);
                x += 1;
            }
            ctx.stroke();
            ctx.restore();
        }
    }

    Timer {
        interval: 16
        running: true
        repeat: true
        onTriggered: {
            compartment.stepForward()
            plot.addPoint(compartment.voltage)
        }
    }
}
