#include "h5_api.h"

int h5_dependency_delta(void);
int h5_object_delta(void);
int h5_shared_delta(void);

h5_payload h5_roundtrip(h5_payload type) {
    int dependency_delta =
        h5_dependency_delta() + h5_object_delta() + h5_shared_delta();
    type.boolean = !type.boolean;
    type.plain_character += 1;
    type.signed_character -= 2;
    type.unsigned_character += 3;
    type.signed_short -= 4;
    type.unsigned_short += 5;
    type.signed_int += 6;
    type.unsigned_int += 7;
    type.signed_long -= 8;
    type.unsigned_long += 9;
    type.signed_long_long -= 10;
    type.unsigned_long_long += 11;
    type.single_precision += 0.5f;
    type.double_precision += 1.25;
    type.fixed_values[0] += 12;
    type.fixed_values[1] += 13;
    type.fixed_values[2] += 14;
    if (type.nullable != 0 || type.opaque != 0) {
        type.boolean = 0;
    }
    type.mode = H5_MODE_POSITIVE;
    type.choice.integer += 16;
    if (type.callback != 0) {
        type.callback(&type.signed_int, dependency_delta);
    }
    type.match += 17;
    type.crate ^= 0x55aa;
    return type;
}

long double h5_long_double(long double value) { return value + 1.0L; }

__int128 h5_int128(__int128 value) { return value + 1; }

double _Complex h5_complex(double _Complex value) { return value; }

#if defined(GERC_H5_ENABLE_BIT_INT)
_BitInt(17) h5_bit_int(_BitInt(17) value) { return value + 1; }
#endif

h5_bits h5_bitfields(h5_bits value) {
    value.low = (value.low + 1u) & 7u;
    return value;
}

int h5_variadic(int count, ...) { return count; }

#if defined(GERC_H5_ENABLE_MS_ABI)
int __attribute__((ms_abi)) h5_msabi(int value) { return value + 1; }
#endif

_Thread_local int h5_tls = 19;
