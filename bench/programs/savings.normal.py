# trace: D155, D159. The savings of `savings.normal.seal` in Python's `decimal`, set to decimal128's rules.
import decimal
from decimal import Decimal
decimal.setcontext(decimal.Context(prec=34, Emax=6144, Emin=-6143, rounding=decimal.ROUND_HALF_EVEN, clamp=1))
RATE = Decimal("1.00001")
CENT = Decimal("0.01")
def run(n):
    savings = Decimal("100.00")
    for _ in range(n):
        savings = savings * RATE + CENT
    return int(savings)
