#ifndef ENTITY_H
#define ENTITY_H

#include <QQuickItem>

class Entity : public QQuickItem
{
    Q_OBJECT
public:
    explicit Entity(QQuickItem* parent = 0);
    ~Entity();

    bool hasFired();
    void setHasFired(bool arg);

signals:
    void stepped(double dt);
    void fired();
    void stimulated(double stimulation);
    void finalizedStep(double dt);

public slots:
    void step(double dt);
    void fire();
    void stimulate(double stimulation);
    void finalizeStep(double dt);

protected:
    virtual void stepEvent(double dt);
    virtual void fireEvent();
    virtual void stimulateEvent(double stimulation);
    virtual void finalizeStepEvent(double dt);

private:
    bool m_hasFired = false;
};

#endif // ENTITY_H
