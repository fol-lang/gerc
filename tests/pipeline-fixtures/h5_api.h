#ifndef GERC_H5_API_H
#define GERC_H5_API_H

#define H5_INTEGER_MACRO 23
#define H5_STRING_MACRO "quote=\" slash=\\ newline=\n"
#define H5_FUNCTION_MACRO(value) ((value) + H5_INTEGER_MACRO)

struct h5_opaque;

typedef enum h5_mode {
    H5_MODE_NEGATIVE = -3,
    H5_MODE_POSITIVE = 7
} h5_mode;

typedef union h5_choice {
    int integer;
    double floating;
} h5_choice;

typedef void (*h5_callback)(int *value, int delta);
typedef void h5_handler(int *value, int delta);

typedef struct h5_payload {
    _Bool boolean;
    char plain_character;
    signed char signed_character;
    unsigned char unsigned_character;
    short signed_short;
    unsigned short unsigned_short;
    int signed_int;
    unsigned int unsigned_int;
    long signed_long;
    unsigned long unsigned_long;
    long long signed_long_long;
    unsigned long long unsigned_long_long;
    float single_precision;
    double double_precision;
    int fixed_values[3];
    void *nullable;
    struct h5_opaque *opaque;
    h5_mode mode;
    h5_choice choice;
    h5_callback callback;
    h5_handler *handler;
    int match;
    int crate;
} h5_payload;

typedef struct h5_bits {
    unsigned int low : 3;
    unsigned int high : 5;
} h5_bits;

#if defined(GERC_H5_ENABLE_VECTOR)
typedef int h5_vector __attribute__((vector_size(16)));
#endif

h5_payload h5_roundtrip(h5_payload type);
long double h5_long_double(long double value);
__int128 h5_int128(__int128 value);
double _Complex h5_complex(double _Complex value);
#if defined(GERC_H5_ENABLE_BIT_INT)
_BitInt(17) h5_bit_int(_BitInt(17) value);
#endif
h5_bits h5_bitfields(h5_bits value);

/* libuv's shape: a record whose callback field takes a pointer to its own
 * alias, held by value inside another record. */
typedef struct h5_signal h5_signal;
typedef void (*h5_signal_cb)(h5_signal *self);
struct h5_signal { h5_signal_cb cb; int count; };
typedef struct h5_loop { h5_signal watcher; int live; } h5_loop;
int h5_loop_count(h5_loop loop);

/* libsodium's hash state, packed like its header packs it: a record whose
 * declared alignment exceeds every field's, beside one packing narrows. */
#pragma pack(push, 1)
typedef struct __attribute__((aligned(64))) h5_hash { unsigned char opaque[96]; } h5_hash;
struct h5_packed { char tag; int value; };
#pragma pack(pop)
int h5_hash_first(const h5_hash *state, const struct h5_packed *packed);
int h5_variadic(int count, ...);
#if defined(GERC_H5_ENABLE_MS_ABI)
int __attribute__((ms_abi)) h5_msabi(int value);
#endif
extern _Thread_local int h5_tls;
#if defined(GERC_H5_ENABLE_VECTOR)
h5_vector h5_vector_value(h5_vector value);
#endif
struct h5_opaque h5_opaque_value(struct h5_opaque value);

int h5_missing(int value);
int h5_hidden(int value);
int h5_weak(int value);
int h5_duplicate(int value);
int h5_ambiguous(int value);
int h5_wrong_target(int value);

#endif
