#ifndef GABORKERNELENGINE_H
#define GABORKERNELENGINE_H

#include "abstractkernelengine.h"

class GaborKernelEngine : public AbstractKernelEngine
{
public:
    GaborKernelEngine();

    // AbstractKernelEngine interface
public:
    virtual void createKernel(vector<vector<double> > *spatial);

private:
    double advance(int x, int y);
};

#endif // GABORKERNELENGINE_H
