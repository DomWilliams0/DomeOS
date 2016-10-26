#ifndef __KERNEL_TESTING_H__
#define __KERNEL_TESTING_H__

extern int printf(const char *, ...);
extern void exit(int);

#define TEST(name, assertion) TEST_WRAPPER(name, assertion, __FILE__, __LINE__)
#define TEST_WRAPPER(name, assertion, filename, lineno)\
	if (!(assertion))\
	{\
		printf("[FAIL] %s (%s) %s:%d\n", #name, #assertion, filename, lineno);\
		exit(1);\
	}\
	else\
	{\
		printf("[PASS] %s\n", #name);\
	}

#endif

void test_strings();
