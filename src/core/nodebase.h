#ifndef NODEBASE_H
#define NODEBASE_H

#include "neuronifyobject.h"

#include <QQuickItem>

class NodeEngine;
class EdgeBase;
class GraphEngine;
class NodeBase : public NeuronifyObject
{
    Q_OBJECT
    Q_PROPERTY(NodeEngine* engine READ engine WRITE setEngine NOTIFY engineChanged)
    Q_PROPERTY(bool inhibitory READ inhibitory WRITE setInhibitory NOTIFY inhibitoryChanged)

public:
    explicit NodeBase(QQuickItem* parent = 0);
    ~NodeBase();

    NodeEngine* engine() const;
    void reset();
    bool inhibitory() const;

signals:
    void edgeAdded(EdgeBase* edge);
    void edgeRemoved(EdgeBase* edge);
    void engineChanged(NodeEngine* arg);
    void inhibitoryChanged(bool inhibitory);

public slots:
    void setEngine(NodeEngine* arg);
    void setInhibitory(bool inhibitory);

private:
    NodeEngine* m_engine = nullptr;
    bool m_inhibitory = false;
    friend class EdgeBase;
};

#endif // NODEBASE_H
