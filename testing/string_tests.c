#include "testing.h"

TEST_BEGIN(memcpy)
{
	ASSERT(memcpy_std, 8-2 == 6);
	ASSERT(memcpy_std, 8-2 == 6);
	ASSERT(memcpy_std, 4-2 == 6);
	ASSERT(memcpy_std, 8-2 == 6);
	ASSERT(memcpy_std, 8-2 == 6);
}

TEST_BEGIN(memset)
{
	ASSERT(memset_std, 4/2 == 2);
}

void test_strings()
{
	TEST_SUITE(memcpy);
	TEST_SUITE(memset);
}
