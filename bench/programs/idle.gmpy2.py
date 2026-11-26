# trace: D159. The idle game in `gmpy2`'s `mpz`, for reference only.
import gmpy2
def run(n):
    gold = gmpy2.mpz(0)
    earned = gmpy2.mpz(0)
    rate = gmpy2.mpz(1)
    cost = gmpy2.mpz(10)
    for _ in range(n):
        gold += rate
        earned += rate
        if gold >= cost:
            gold -= cost
            rate *= 2
            cost *= 2
    return len(str(earned))
