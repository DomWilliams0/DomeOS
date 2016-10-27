#ifndef __KERNEL_TESTS_H__
#define __KERNEL_TESTS_H__

extern int printf(const char *, ...);
extern int puts(const char *);
extern void exit(int);

#define ASSERT(name, assertion) ASSERT_WRAPPER(name, assertion, __FILE__, __LINE__)
#define ASSERT_WRAPPER(name, assertion, filename, lineno)\
	*count += 1;\
	if (!(assertion))\
	{\
		printf("[FAIL] %s (%s) %s:%d\n", #name, #assertion, filename, lineno);\
		*err += 1;\
	}\
	else\
	{\
		printf("[PASS] %s\n", #name);\
	}

#endif

#define TEST_BEGIN(suite_name)\
	static void test_ ## suite_name(int *count, int *err)


#define TEST_SUITE(name)\
{\
	int count = 0;\
	int err = 0;\
	test_ ## name(&count, &err);\
	printf("[SUITE %s] %s passed %d/%d tests\n", err == 0 ? "PASS" : "FAIL", #name, count - err, count);\
	puts("===================");\
}

void test_strings();
void test_errors();
