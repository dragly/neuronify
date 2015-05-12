#include "retinapainter.h"
#include <QPainter>

RetinaPainter::RetinaPainter()
{

}

void RetinaPainter::paint(QPainter *painter)
{
    if(!m_retinaEngine) {
        return;
    }
    if (m_retinaEngine->plotReceptiveField()){
        if(m_retinaEngine->receptiveField()){
            painter->drawImage(boundingRect(), m_retinaEngine->receptiveField()->spatialImage());
            update();
        }
    } else {
        painter->drawImage(boundingRect(), m_retinaEngine->paintedImage());
        update();
    }
}

void RetinaPainter::setRetinaEngine(RetinaEngine *retinaEngine)
{
    if (m_retinaEngine == retinaEngine)
        return;

    m_retinaEngine = retinaEngine;
    emit retinaEngineChanged(retinaEngine);
}

RetinaEngine *RetinaPainter::retinaEngine() const
{
    return m_retinaEngine;
}

