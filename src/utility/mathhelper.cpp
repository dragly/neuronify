#include "mathhelper.h"

MathHelper::MathHelper()
{

}

double MathHelper::heaviside(double x)
{
    if (x < 0){
        return 0;
    }else{
        return 1;
    }
}


