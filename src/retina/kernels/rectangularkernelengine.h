#ifndef OFFLEFTKERNELENGINE_H
#define OFFLEFTKERNELENGINE_H

#include "abstractkernelengine.h"



class OffLeftKernelEngine : public AbstractKernelEngine
{
public:
    OffLeftKernelEngine();

    // AbstractKernelEngine interface
public:
    virtual void createKernel(vector<vector<double> > *spatial);
};

#endif // OFFLEFTKERNELENGINE_H
