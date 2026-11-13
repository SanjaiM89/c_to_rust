/*
 * differential_c.c - reference driver for the C implementation.
 *
 * Reads one JSON document on stdin and prints a canonical transcript of
 * cJSON's observable behaviour. The Rust driver must produce a byte-identical
 * transcript for the same input; any difference is a divergence.
 *
 * The transcript covers: parse success, error offset, the parsed tree
 * (canonical, byte-exact), formatted and unformatted printing, minifying,
 * and a print/parse round trip.
 *
 * Build: gcc -I vendor/cJSON differential_c.c vendor/cJSON/cJSON.c -o diff_c
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "cJSON.h"

/* print bytes as lowercase hex so the transcript is unambiguous */
static void print_hex(const unsigned char *s, size_t n)
{
    size_t i;
    for (i = 0; i < n; i++)
    {
        printf("%02x", s[i]);
    }
}

static void print_tree(const cJSON *item, int depth)
{
    const cJSON *child;
    int i;

    for (i = 0; i < depth; i++) { printf(" "); }
    printf("N");
    printf(" t=%d", item ? item->type : -1);
    if (item == NULL) { printf("\n"); return; }
    printf(" i=%d", item->valueint);
    printf(" d=%.17g", item->valuedouble);
    printf(" k=");
    print_hex((const unsigned char *)(item->string ? item->string : (const char *)""),
              item->string ? strlen(item->string) : 0);
    printf(" v=");
    print_hex((const unsigned char *)(item->valuestring ? item->valuestring : (const char *)""),
              item->valuestring ? strlen(item->valuestring) : 0);
    printf("\n");
    for (child = item->child; child != NULL; child = child->next)
    {
        print_tree(child, depth + 1);
    }
}

int main(void)
{
    /* read all of stdin */
    size_t cap = 65536, len = 0;
    char *buf = (char *)malloc(cap);
    size_t n;
    cJSON *root;
    const char *err;
    char *out, *out_unf;
    char *copy;
    const char *parse_end = NULL;

    if (!buf) { return 2; }
    while ((n = fread(buf + len, 1, cap - len - 1, stdin)) > 0)
    {
        len += n;
        if (len + 1 >= cap)
        {
            char *nb = (char *)realloc(buf, cap * 2);
            if (!nb) { free(buf); return 2; }
            buf = nb;
            cap *= 2;
        }
    }
    buf[len] = '\0';

    printf("INPUT_LEN %zu\n", len);

    /* ---- parse ---- */
    root = cJSON_ParseWithOpts(buf, &parse_end, 0);
    if (root == NULL)
    {
        err = cJSON_GetErrorPtr();
        printf("STATUS fail\n");
        printf("ERROR_OFFSET %ld\n", err ? (long)(err - buf) : -1L);
        printf("TREE\n");
        printf("PRINT\n");
        printf("PRINT_UNFORMATTED\n");
        printf("MINIFY\n");
        printf("ROUNDTRIP none\n");
        free(buf);
        return 0;
    }
    printf("STATUS ok\n");
    printf("ERROR_OFFSET %ld\n", (long)(parse_end - buf));

    /* ---- tree ---- */
    printf("TREE\n");
    print_tree(root, 0);

    /* ---- printing ---- */
    out = cJSON_Print(root);
    out_unf = cJSON_PrintUnformatted(root);
    printf("PRINT ");
    if (out) { fputs(out, stdout); } else { fputs("(null)", stdout); }
    printf("\n");
    printf("PRINT_UNFORMATTED ");
    if (out_unf) { fputs(out_unf, stdout); } else { fputs("(null)", stdout); }
    printf("\n");

    /* ---- minify (on a copy) ---- */
    copy = (char *)malloc(len + 2);
    if (copy)
    {
        memcpy(copy, buf, len + 1);
        cJSON_Minify(copy);
        printf("MINIFY ");
        fputs(copy, stdout);
        printf("\n");
        free(copy);
    }
    else
    {
        printf("MINIFY\n");
    }

    /* ---- round trip: reparse the printed output and compare trees ---- */
    if (out_unf)
    {
        cJSON *again = cJSON_Parse(out_unf);
        printf("ROUNDTRIP %s\n", (again != NULL) ? "ok" : "fail");
        if (again != NULL)
        {
            char *out2 = cJSON_PrintUnformatted(again);
            printf("ROUNDTRIP_EQUAL %s\n",
                   (out2 && strcmp(out2, out_unf) == 0) ? "yes" : "no");
            if (out2) { cJSON_free(out2); }
            cJSON_Delete(again);
        }
        else
        {
            printf("ROUNDTRIP_EQUAL no\n");
        }
    }
    else
    {
        printf("ROUNDTRIP none\n");
        printf("ROUNDTRIP_EQUAL no\n");
    }

    if (out) { cJSON_free(out); }
    if (out_unf) { cJSON_free(out_unf); }
    cJSON_Delete(root);
    free(buf);
    return 0;
}
