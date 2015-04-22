#ifndef EDGE_H
#define EDGE_H

#include <QQuickItem>

class Node;
class Edge : public QQuickItem
{
    Q_OBJECT
    Q_PROPERTY(Node* itemA READ itemA WRITE setItemA NOTIFY itemAChanged)
    Q_PROPERTY(Node* itemB READ itemB WRITE setItemB NOTIFY itemBChanged)

public:
    explicit Edge(QQuickItem *parent = 0);
    ~Edge();

    Node* itemA() const;
    Node* itemB() const;

signals:
    void itemAChanged(Node* arg);
    void itemBChanged(Node* arg);

public slots:
    void setItemA(Node* arg);
    void setItemB(Node* arg);

private:
    Node* m_itemA = nullptr;
    Node* m_itemB = nullptr;
};

#endif // EDGE_H
