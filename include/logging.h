#ifndef __KERNEL_LOGGING_H__
#define __KERNEL_LOGGING_H__

#define KDEBUG

#ifdef KDEBUG
#define LOG_DEBUG(msg) log("[DEBUG] ", msg)
#else
#define LOG_DEBUG(msg)
#endif

void log(char *prefix, char *message);

#endif

