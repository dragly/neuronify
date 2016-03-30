#ifndef RECTANGULARKERNELENGINE_H
#define RECTANGULARKERNELENGINE_H

#include "abstractkernelengine.h"



class RectangularKernelEngine : public AbstractKernelEngine
{
    Q_OBJECT
    Q_PROPERTY(double orientation READ orientation WRITE setOrientation NOTIFY orientationChanged)
public:
    RectangularKernelEngine();

    // AbstractKernelEngine interface
public:
    virtual void createKernel(vector<vector<double> > *spatial);
    double orientation() const;

public slots:
    void setOrientation(double orientation);

signals:
    void orientationChanged(double orientation);

private:
    double advance(double y);
    double m_orientation = 0.0;
};

#endif // RECTANGULARKERNELENGINE_H
