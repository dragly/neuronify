#ifndef RECTANGULARKERNELENGINE_H
#define RECTANGULARKERNELENGINE_H

#include "abstractkernelengine.h"



class RectangularKernelEngine : public AbstractKernelEngine
{
public:
    RectangularKernelEngine();

    // AbstractKernelEngine interface
public:
    virtual void createKernel(vector<vector<double> > *spatial);
};

#endif // RECTANGULARKERNELENGINE_H
