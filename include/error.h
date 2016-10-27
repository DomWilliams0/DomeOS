#ifndef __KERNEL_ERROR_H__
#define __KERNEL_ERROR_H__

// ERR_SUCCESS is 0 and will evaluate to false, allowing for code such as:
// if (peek_error()) { ... deal with error ... }

#define error(err) set_error(err, __FILE__, __FUNCTION__, __LINE__)

#define ERR_SUCCESS 0x000 // no error
#define ERR_INPUT   0x101 // bad input

typedef unsigned int err;

struct err_state
{
    err err;
    const char *file;
    const char *func;
    unsigned int line;
};

extern struct err_state last_error;

// sets last_error
void set_error(err e, const char *file, const char *function, unsigned int line);

// returns last_error state after clearing it
struct err_state get_error();

// returns just the last error code after clearing it
err get_error_code();

// returns last_error without clearing it
err peek_error();

// prints the given error
void print_error(struct err_state *error);

// converts the given error code into a string
void get_error_str(err err, char **str);

#endif

