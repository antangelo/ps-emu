int factorial(int n)
{
    if (n == 0) return 1;

    return n * factorial(n - 1);
}

int fibonnaci(int n)
{
    if (n == 1) return 1;
    if (n == 2) return 1;

    return fibonnaci(n - 1) + fibonnaci(n - 2);
}

void print(char *c)
{
    volatile char *printer = (volatile char*)0x1fd003f8;
    while (*c) {
        *printer = *c;
        ++c;
    }
}

int main()
{
    int f = fibonnaci(5);

    //*printer = f; 
    print("Hello world!\r\n");

    return 0;
}

