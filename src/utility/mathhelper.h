#ifndef MATHHELPER_H
#define MATHHELPER_H

#include <iostream>
using namespace std;

const static long double pi = 3.141592653589793238462643383279502884L;

class MathHelper
{
public:
    MathHelper();

    static double heaviside(double x);
};

#endif // MATHHELPER_H
