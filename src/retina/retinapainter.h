#ifndef RETINAPAINTER_H
#define RETINAPAINTER_H
#include <QQuickPaintedItem>

#include "retinaengine.h"

class RetinaPainter :  public QQuickPaintedItem
{
    Q_OBJECT
    Q_PROPERTY(RetinaEngine *retinaEngine READ retinaEngine WRITE setRetinaEngine NOTIFY retinaEngineChanged)

    RetinaEngine * m_retinaEngine = nullptr;

public:
    RetinaPainter();
    RetinaEngine *retinaEngine() const;

    // QQuickPaintedItem interface
    virtual void paint(QPainter *painter);


public slots:
    void setRetinaEngine(RetinaEngine * retinaEngine);
signals:
    void retinaEngineChanged(RetinaEngine * retinaEngine);
};

#endif // RETINAPAINTER_H
