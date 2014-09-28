import QtQuick 2.0

Item {
    signal dragStarted
    property vector2d velocity
    property bool dragging: false
}
