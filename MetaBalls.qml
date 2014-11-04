import QtQuick 2.0

Item {
    width: 640
    height: 480

    ShaderEffect {
        id: shaderEffect
        anchors.fill: parent
        blending: true
        property vector2d ball1: Qt.vector2d(ballItem1.x + ballItem1.width / 2.0, ballItem1.y + ballItem1.height / 2.0)
        property vector2d ball2: Qt.vector2d(ballItem2.x + ballItem2.width / 2.0, ballItem2.y + ballItem2.height / 2.0)
        property vector2d ball3: Qt.vector2d(ballItem3.x + ballItem3.width / 2.0, ballItem3.y + ballItem3.height / 2.0)
        property vector2d shaderPosition: Qt.vector2d(x, y)
        property vector2d shaderSize: Qt.vector2d(width, height)

        fragmentShader: "
        #version 330
        out vec4 gl_FragColor;
        in vec2 qt_TexCoord0;
        uniform vec2 shaderPosition;
        uniform vec2 shaderSize;
        uniform vec2 ball1;
        uniform vec2 ball2;
        uniform vec2 ball3;
        float test(vec2 dist) {
            float squared = dist.x * dist.x + dist.y  * dist.y;
            return length(dist);
        }
        void main() {
            float ballSize1 = 8;
            float ballSize2 = 3;
            vec4 pixelColor = vec4(0.0, 0.0, 0.0, 0.0);
            vec4 color = vec4(0.0, 0.5, 1.0, 1.0);
            vec4 color2 = vec4(1.0, 0.6, 1.0, 1.0);

            vec2 coords = qt_TexCoord0;
            coords -= shaderPosition;
            coords.x *= shaderSize.x;
            coords.y *= shaderSize.y;

            vec2 dist;
            float val;
            dist = ball1 - coords.xy;
            val = (ballSize1*ballSize1) / test(dist);
            pixelColor += color2 * val;
            dist = ball2 - coords.xy;
            val = (ballSize2*ballSize2) / test(dist);
            pixelColor += color2 * val;
            dist = ball3 - coords.xy;
            val = (ballSize2*ballSize2) / test(dist);
            pixelColor += color2 * val;

            float a = smoothstep(0.99, 1.0, pixelColor.a);
            gl_FragColor = vec4(pixelColor * a);
        }
"
    }

    Rectangle {
        id: ballItem1
        x: 100
        y: 200
        opacity: 0.0
        width: 50
        height: width
        color: "blue"
        MouseArea {
            anchors.fill: parent
            drag.target: parent
        }
    }

    Rectangle {
        id: ballItem2
        x: 300
        y: 200
        opacity: 0.0
        width: 50
        height: width
        color: "blue"
        MouseArea {
            anchors.fill: parent
            drag.target: parent
        }
    }

    Rectangle {
        id: ballItem3
        x: 300
        y: 300
        opacity: 0.0
        width: 50
        height: width
        color: "blue"
        MouseArea {
            anchors.fill: parent
            drag.target: parent
        }
    }
}
