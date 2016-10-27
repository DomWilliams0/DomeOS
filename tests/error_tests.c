#include <string.h>

#include "tests.h"
#include "error.h"

TEST_BEGIN(error)
{
    ASSERT(default, peek_error() == ERR_SUCCESS);

    error(ERR_INPUT);
    ASSERT(peek, peek_error() == ERR_INPUT);
    ASSERT(get_and_clear, get_error() == ERR_INPUT && peek_error() == ERR_SUCCESS);

    error(ERR_INPUT);
    struct err_state state = get_error_state();
    ASSERT(state,
            state.err == ERR_INPUT &&
            strcmp(state.file, __FILE__) == 0 &&
            state.line == __LINE__ - 5);
            // strcmp(state.func, __FUNCTION__) == 0);

    char *err_string = NULL;
    get_error_str(state.err, &err_string);
    ASSERT(error_string, err_string && strcmp(err_string, "ERR_INPUT") == 0);
}

void test_errors()
{
    TEST_SUITE(error);
}

