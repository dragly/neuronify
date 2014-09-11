import QtQuick 2.0

Rectangle {
    property real targetVoltage: 0.0
    property bool forceTargetVoltage: false

    property real voltage: 0.0
    property real meanSodiumConductance: 120
    property real meanPotassiumConductance: 36
    property real meanLeakConductance: 0.3
    property real cm: 1.0
    property real sodiumPotential: 50
    property real potassiumPotential: -77
    property real leakPotential: -54.4
    property real sodiumActivation: 0.0
    property real sodiumActivationAlpha: 0.1 * (voltage + 40) / (1 - Math.exp(-(voltage+40/10)))
    property real sodiumActivationBeta: 4 * Math.exp(-(voltage + 65) / 18.0)
    property real sodiumInactivation: 0.0
    property real sodiumInactivationAlpha: 0.07 * Math.exp(-(voltage + 65) / 20.0)
    property real sodiumInactivationBeta: 1.0 / (Math.exp(-(voltage + 35)/10) + 1.0)
    property real potassiumActivation: 0.0
    property real potassiumActivationAlpha: 0.01 * (voltage + 55) / (1 - Math.exp(-(voltage + 55) / 10.0))
    property real potassiumActivationBeta: 0.125 * Math.exp(- (voltage + 65) / 80)
    property real dt: 0.01

    function stepForward() {
        var m = sodiumActivation
        var alpham = sodiumActivationAlpha
        var betam = sodiumActivationBeta
        var dm = dt * (alpham * (1 - m) - betam * m)
        var h = sodiumInactivation
        var alphah = sodiumInactivationAlpha
        var betah = sodiumInactivationBeta
        var dh = dt * (alphah * (1 - h) - betah * h)
        var n = potassiumActivation
        var alphan = potassiumActivationAlpha
        var betan = potassiumActivationBeta
        var dn = dt * (alphan * (1 - n) - betan * n)

        sodiumActivation += dm
        sodiumInactivation += dh
        potassiumActivation += dn

        if(forceTargetVoltage) {
            voltage = targetVoltage
        } else {
            var gL = meanLeakConductance
            var gNa = meanSodiumConductance
            var gK = meanPotassiumConductance
            var EL = leakPotential
            var ENa = sodiumPotential
            var EK = potassiumPotential
            var V = voltage
            var m3 = m*m*m
            var n4 = n*n*n*n
            var dV = dt * ((1.0 / cm) * (- gL * (V - EL) - gNa * m3 * h * (V - ENa) - gK * n4 * (V - EK)))
            voltage += dV
        }
    }

    color: "red"

    width: 100
    height: 62

    Text {
        text: voltage
    }
}
