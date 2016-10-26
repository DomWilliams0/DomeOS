#include "testing.h"

static void test_memcpy()
{
	TEST(memcpy_std, 8-2 == 6);
}

static void test_memset()
{
	TEST(memset_std, 4/2 == 2);
}

void test_strings()
{
	test_memcpy();
	test_memset();
}
