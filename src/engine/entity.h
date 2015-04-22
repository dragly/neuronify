#ifndef ENTITY_H
#define ENTITY_H

#include <QQuickItem>

class Entity : public QQuickItem
{
    Q_OBJECT
public:
    explicit Entity(QQuickItem* parent = 0);
    ~Entity();

signals:
    void stepped(double dt);
    void fired();
    void stimulated(double stimulation);

public slots:
    void step(double dt);
    void fire();
    void stimulate(double stimulation);

protected:
    virtual void stepEvent(double dt);
    virtual void fireEvent();
    virtual void stimulateEvent(double stimulation);

private:
};

#endif // ENTITY_H
