#ifndef KERNELABSTRACTENGINE_H
#define KERNELABSTRACTENGINE_H

#include <QQuickItem>
#include <vector>
#include <iostream>
#include "../../core/neuronifyobject.h"
#include "../../utility/mathhelper.h"

using namespace std;


class AbstractKernelEngine: public NeuronifyObject
{
    Q_OBJECT
    Q_PROPERTY(int resolutionHeight READ resolutionHeight WRITE setResolutionHeight NOTIFY resolutionHeightChanged)
    Q_PROPERTY(int resolutionWidth READ resolutionWidth WRITE setResolutionWidth NOTIFY resolutionWidthChanged)
    Q_PROPERTY(double shift_x READ shift_x WRITE setShift_x NOTIFY shift_xChanged)
    Q_PROPERTY(double shift_y READ shift_y WRITE setShift_y NOTIFY shift_yChanged)


public:
    AbstractKernelEngine();

    virtual void createKernel(vector<vector<double> > * spatial) = 0;
    int resolutionHeight() const;
    int resolutionWidth() const;

    vector<double> x() const;
    vector<double> y() const;

    double shift_x() const;
    double shift_y() const;

public slots:
    void setResolutionHeight(int resolutionHeight);
    void setResolutionWidth(int resolutionWidth);

    void setShift_x(double shift_x);
    void setShift_y(double shift_y);

signals:
    void resolutionHeightChanged(int resolutionHeight);
    void resolutionWidthChanged(int resolutionWidth);
    void needsRecreation();
    void shift_xChanged(double shift_x);
    void shift_yChanged(double shift_y);

protected:
    int m_resolutionHeight=20;
    int m_resolutionWidth=20;
    vector<double> m_x;
    vector<double> m_y;
    double m_shift_x;
    double m_shift_y;

private:
    void updateX();
    void updateY();
};

#endif // KERNELABSTRACTENGINE_H
