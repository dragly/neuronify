#ifndef NODEENGINE_H
#define NODEENGINE_H

#include <QQuickItem>

class NodeEngine : public QQuickItem
{
    Q_OBJECT
    Q_PROPERTY(double stimulation READ stimulation WRITE setStimulation NOTIFY stimulationChanged)

public:
    explicit NodeEngine(QQuickItem *parent = 0);
    ~NodeEngine();

    double stimulation() const;

    bool hasFired();
    void setHasFired(bool arg);

signals:
    void stepped(double dt);
    void fired();
    void stimulated(double stimulation);
    void finalizedStep(double dt);
    void stimulationChanged(double arg);

public slots:
    void step(double dt);
    void fire();
    void stimulate(double stimulation);
    void finalizeStep(double dt);
    void setStimulation(double arg);

protected:
    virtual void stepEvent(double dt);
    virtual void fireEvent();
    virtual void stimulateEvent(double stimulation);
    virtual void finalizeStepEvent(double dt);

private:
    bool m_hasFired = false;
    double m_stimulation = 0.0;
};

#endif // NODEENGINE_H
