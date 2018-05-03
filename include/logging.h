#ifndef __KERNEL_LOGGING_H__
#define __KERNEL_LOGGING_H__

#define KDEBUG

#ifdef KDEBUG
#define LOG_DEBUG(msg) log("[DEBUG] ", msg)
#else
#define LOG_DEBUG(msg)
#endif

char* itoa(int value, char* result, int base);

void log(char *prefix, char *message);

void log_raw(char *message);

#endif

