# trace: D159. The same factorial in `gmpy2`'s `mpz`, GMP's whole numbers, for reference only.
import gmpy2
def run(n):
    product = gmpy2.mpz(1)
    for i in range(2, n + 1):
        product *= i
    return len(str(product))
