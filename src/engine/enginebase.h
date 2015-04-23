#ifndef NEURONIFY_ENGINEBASE_H
#define NEURONIFY_ENGINEBASE_H

#include <QQuickItem>

class EngineBase : public QQuickItem
{
    Q_OBJECT
public:
    explicit EngineBase(QQuickItem* parent = 0);
    ~EngineBase();

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

#endif // NEURONIFY_ENGINEBASE_H
