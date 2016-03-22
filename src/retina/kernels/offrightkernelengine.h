#ifndef OFFRIGHTKERNELENGINE_H
#define OFFRIGHTKERNELENGINE_H

#include "abstractkernelengine.h"



class OffRightKernelEngine : public AbstractKernelEngine
{
public:
    OffRightKernelEngine();

    // AbstractKernelEngine interface
public:
    virtual void createKernel(vector<vector<double> > *spatial);
};

#endif // OFFRIGHTKERNELENGINE_H
