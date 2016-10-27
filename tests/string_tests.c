#include "tests.h"
#include "util/string.h"

TEST_BEGIN(kmemcmp)
{
	char *a = "aaaa";
	char *b = "bbbb";

	ASSERT(std, kmemcmp(a, a, 4));
	ASSERT(different, !kmemcmp(a, b, 4));
	ASSERT(null, kmemcmp(a, b, 0));
}

TEST_BEGIN(kmemcpy)
{
	char a[] = "abcdefgh";
	char b[] = "stuvwxyz";

	kmemcpy(b, a, 4);
	ASSERT(subcpy, kmemcmp(b, "abcdwxyz", 8));

	kmemcpy(a, b, 8);
	ASSERT(fullcpy, kmemcmp(a, "abcdwxyz", 8));

	kmemcpy(a, "xxxxxxxx", 0);
	ASSERT(nullcpy, kmemcmp(a, "abcdwxyz", 8));

	unsigned short wide_a[] = {15, 20, 25};
	unsigned short wide_b[] = {10000, 2, 3, 4, 5, 6};

	kwmemcpy(wide_b, wide_a, 1);
	ASSERT(wide, wide_b[0] == 15);
}

TEST_BEGIN(kmemset)
{
	char a[4] = "abcd";

	kmemset(a, 'a', 4);
	ASSERT(std, kmemcmp(a, "aaaa", 4));

	kmemset(a, 'b', 0);
	ASSERT(null, kmemcmp(a, "aaaa", 4));

	short wide[] = {1, 2, 3};
	kwmemset(wide, 10000, 3);
	ASSERT(wide, wide[2] == 10000);
}

void test_strings()
{
	TEST_SUITE(kmemcmp);
	TEST_SUITE(kmemcpy);
	TEST_SUITE(kmemset);
}
