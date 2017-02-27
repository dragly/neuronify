#ifndef EDGEBASE_H
#define EDGEBASE_H

#include <QQuickItem>
#include <QPointer>

#include "neuronifyobject.h"

class NodeBase;
class EdgeEngine;

class EdgeBase : public NeuronifyObject
{
    Q_OBJECT
    Q_PROPERTY(NodeBase* itemA READ itemA WRITE setItemA NOTIFY itemAChanged)
    Q_PROPERTY(NodeBase* itemB READ itemB WRITE setItemB NOTIFY itemBChanged)
    Q_PROPERTY(EdgeEngine* engine READ engine WRITE setEngine NOTIFY engineChanged)
    Q_PROPERTY(bool curved READ curved WRITE setCurved NOTIFY curvedChanged)

public:
    explicit EdgeBase(QQuickItem *parent = 0);

    NodeBase* itemA() const;
    NodeBase* itemB() const;

    bool curved() const;

    EdgeEngine* engine() const;

signals:
    void itemAChanged(NodeBase* arg);
    void itemBChanged(NodeBase* arg);
    void curvedChanged(bool curved);
    void engineChanged(EdgeEngine* engine);

public slots:
    void setItemA(NodeBase* arg);
    void setItemB(NodeBase* arg);
    void setCurved(bool curved);
    void setEngine(EdgeEngine* engine);

private:
    QPointer<NodeBase> m_itemA;
    QPointer<NodeBase> m_itemB;
    bool m_curved = false;
    EdgeEngine* m_engine = nullptr;

    friend class GraphEngine;
};

#endif // EDGEBASE_H
