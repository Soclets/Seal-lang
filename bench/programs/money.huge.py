# trace: D155, D159. The sums of `money.huge.wax` in Python's `decimal`, with room for every digit.
import decimal
from decimal import Decimal
decimal.setcontext(decimal.Context(prec=decimal.MAX_PREC, Emax=decimal.MAX_EMAX, Emin=decimal.MIN_EMIN, traps=[decimal.Inexact, decimal.Overflow, decimal.InvalidOperation]))
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
