#ifndef NODE_H
#define NODE_H

#include <QQuickItem>

#include "entity.h"

class Node : public Entity
{
    Q_OBJECT
public:
    explicit Node(QQuickItem* parent = 0);
    ~Node();

signals:

public slots:
};

#endif // NODE_H
