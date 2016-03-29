#ifndef EDGE_H
#define EDGE_H

#include <QQuickItem>
#include <QPointer>


class NodeBase;
class Edge : public QQuickItem
{
    Q_OBJECT
    Q_PROPERTY(NodeBase* itemA READ itemA WRITE setItemA NOTIFY itemAChanged)
    Q_PROPERTY(NodeBase* itemB READ itemB WRITE setItemB NOTIFY itemBChanged)
    Q_PROPERTY(int curved READ curved WRITE setCurved NOTIFY curvedChanged)

public:
    explicit Edge(QQuickItem *parent = 0);

    NodeBase* itemA() const;
    NodeBase* itemB() const;

    int curved() const
;

signals:
    void itemAChanged(NodeBase* arg);
    void itemBChanged(NodeBase* arg);

    void curvedChanged(bool curved);

public slots:
    void setItemA(NodeBase* arg);
    void setItemB(NodeBase* arg);

    void setCurved(int curved);

private:
    QPointer<NodeBase> m_itemA;
    QPointer<NodeBase> m_itemB;
    int m_curved;

    friend class GraphEngine;
};

#endif // EDGE_H
