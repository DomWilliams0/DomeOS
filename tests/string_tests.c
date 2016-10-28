#include "tests.h"
#include "util/string.h"

TEST_BEGIN(kmemcmp)
{
	char *a = "aaaa";
	char *b = "aabb";

	ASSERT(std, kmemcmp(a, a, 4));
	ASSERT(different, !kmemcmp(a, b, 4));
	ASSERT(sub, kmemcmp(a, b, 2));
	ASSERT(zero, kmemcmp(a, b, 0));
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

TEST_BEGIN(kuxtos)
{
    char buf[16] = { 0 };
    ksize_t out = 0;

    kuxtos(32, buf, &out);
    ASSERT(simple, out == 4 && kmemcmp(buf, "0x20", out));

    kmemset(buf, '\0', out);
    kuxtos(178298882, buf, &out);
    ASSERT(harder, out == 9 && kmemcmp(buf, "0xAA0A002", out));

    kmemset(buf, '\0', out);
    kuxtos(4294967295, buf, &out);
    ASSERT(limit, out == 10 && kmemcmp(buf, "0xFFFFFFFF", out));

    kmemset(buf, '\0', out);
    kuxtos(0, buf, &out);
    ASSERT(zero, out == 3 && kmemcmp(buf, "0x0", out));
}

TEST_BEGIN(kuitos)
{
    char buf[16] = { 0 };
    ksize_t out = 0;

    kuitos(1, buf, &out);
    ASSERT(one, out == 1 && kmemcmp(buf, "1", out));

    kmemset(buf, '\0', out);
    kuitos(12093090, buf, &out);
    ASSERT(harder, out == 8 && kmemcmp(buf, "12093090", out));

    kmemset(buf, '\0', out);
    kuitos(4294967295, buf, &out);
    ASSERT(limit, out == 10 && kmemcmp(buf, "4294967295", out));

    kmemset(buf, '\0', out);
    kuitos(0, buf, &out);
    ASSERT(zero, out == 1 && kmemcmp(buf, "0", out));
}

void test_strings()
{
	TEST_SUITE(kmemcmp);
	TEST_SUITE(kmemcpy);
	TEST_SUITE(kmemset);

	TEST_SUITE(kuxtos);
	TEST_SUITE(kuitos);
}
