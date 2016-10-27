#include "error.h"

struct err_state last_error = { ERR_SUCCESS, "", 0 };

static void clear_error()
{
    set_error(ERR_SUCCESS, "", 0);
}

void set_error(err e, const char *file,/*const char *function,*/ unsigned int line)
{
    last_error.err = e;
    last_error.file = file;
    last_error.line = line;
}

err get_error()
{
    err e = last_error.err;
    clear_error();
    return e;
}

struct err_state get_error_state()
{
    struct err_state state = last_error;
    clear_error();
    return state;
}

err peek_error()
{
    return last_error.err;
}

void get_error_str(err err, char **str)
{
    // TODO autogenerate lookup table with preprocessor
    switch (err)
    {
        case ERR_SUCCESS:
            *str = "ERR_SUCCESS";
            return;
        case ERR_INPUT:
            *str = "ERR_INPUT";
            return;
        default:
            *str = 0;
            return;
    }
}
