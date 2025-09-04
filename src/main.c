#include <stdio.h>
#include <string.h>
#include "../rust_version/include/rust_version.h"

int main(int argc, char **argv)
{
    if (rust_handle_args(argc, (const char**)argv))
        return 0;
    return 0;
}
