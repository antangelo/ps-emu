//int factorial(int n)
//{
//    if (n == 0) return 1;
//
//    return n * factorial(n - 1);
//}

void waste_time();
int reg_fib(int v);

int fibonnaci(int n)
{
    if (n == 1) return 1;
    if (n == 2) return 1;

    return fibonnaci(n - 1) + fibonnaci(n - 2);
}


void format_uint_to_str(unsigned int num, unsigned int base, char *bf)
{
	int n = 0;
	int dgt;
	unsigned int d = 1;

	while ((num / d) >= base)
		d *= base;
	while (d != 0) {
		dgt = num / d;
		num %= d;
		d /= base;

		if (n || dgt > 0 || d == 0) {
			*bf++ = ((dgt) + ((dgt) < 10 ? '0' : 'a' - 10));
			++n;
		}
	}
	*bf = 0;
}


void print(char *c)
{
    volatile char *printer = (volatile char*)0x1fd003f8;
    while (*c) {
        *printer = *c;
        ++c;
    }
}


void print_int(int d)
{
    char buf[10];
    format_uint_to_str(d, 10, buf);
    print(buf);
}

int main()
{
    int f;

    for (int i = 3; i <= 31; ++i) f = reg_fib(i);
    (void)f;

    //waste_time();

    print_int(f);
    print("\r\n");
    print("Test program!\r\n");

    return 0;
}

