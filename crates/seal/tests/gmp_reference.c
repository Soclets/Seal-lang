/* GMP as an outside reference for Seal's whole numbers of any size (D115), used by tests/gmp.rs only: it reads
 * lines of "operation a b" in decimal and prints each answer in decimal, one line per case. Not part of Seal. */
#include <gmp.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    char *line = NULL;
    size_t room = 0;
    mpz_t a, b, q, r;
    mpz_inits(a, b, q, r, NULL);
    while (getline(&line, &room, stdin) > 0) {
        char *operation = strtok(line, " \n");
        char *first = strtok(NULL, " \n");
        char *second = strtok(NULL, " \n");
        if (!operation || !first || !second) continue;
        mpz_set_str(a, first, 10);
        mpz_set_str(b, second, 10);
        if (strcmp(operation, "multiply") == 0) {
            mpz_mul(q, a, b);
            gmp_printf("%Zd\n", q);
        } else if (strcmp(operation, "add") == 0) {
            mpz_add(q, a, b);
            gmp_printf("%Zd\n", q);
        } else if (strcmp(operation, "floor") == 0 || strcmp(operation, "truncate") == 0) {
            if (mpz_sgn(b) == 0) { printf("by_zero\n"); continue; }
            if (operation[0] == 'f') mpz_fdiv_qr(q, r, a, b); else mpz_tdiv_qr(q, r, a, b);
            gmp_printf("%Zd %Zd\n", q, r);
        } else if (strcmp(operation, "root") == 0) {
            mpz_abs(a, a);
            mpz_sqrt(q, a);
            gmp_printf("%Zd\n", q);
        } else if (strcmp(operation, "power") == 0) {
            mpz_pow_ui(q, a, mpz_get_ui(b));
            gmp_printf("%Zd\n", q);
        } else {
            printf("unknown\n");
        }
    }
    free(line);
    mpz_clears(a, b, q, r, NULL);
    return 0;
}
