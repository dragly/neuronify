#include "retinapainter.h"
#include <QPainter>

RetinaPainter::RetinaPainter()
{

}

void RetinaPainter::paint(QPainter *painter)
{
    painter->drawImage(boundingRect(), m_retinaEngine->image());
    update();
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

