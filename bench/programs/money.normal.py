# trace: D155, D159. The sums of `money.normal.seal` in Python's `decimal`, set to decimal128's rules.
import decimal
from decimal import Decimal
decimal.setcontext(decimal.Context(prec=34, Emax=6144, Emin=-6143, rounding=decimal.ROUND_HALF_EVEN, clamp=1))
PRICE = Decimal("4.99")
TAX_RATE = Decimal("0.08")
def run(n):
    total = Decimal("0.00")
    tax = Decimal("0.00")
    for i in range(n):
        total += PRICE * (i % 7 + 1)
        tax += PRICE * TAX_RATE
    share = (total + tax) // 3
    return int(share)
