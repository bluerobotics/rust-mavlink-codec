#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include <ardupilotmega/mavlink.h>

struct mavlink_codec_bench_state {
    mavlink_message_t rxmsg;
    mavlink_status_t status;
    mavlink_message_t r_message;
    mavlink_status_t r_status;
    mavlink_message_t last_message;
};

struct mavlink_codec_bench_state *mavlink_codec_bench_state_new(void) {
    struct mavlink_codec_bench_state *state =
        calloc(1, sizeof(struct mavlink_codec_bench_state));
    return state;
}

void mavlink_codec_bench_state_reset(struct mavlink_codec_bench_state *state) {
    memset(&state->rxmsg, 0, sizeof(state->rxmsg));
    memset(&state->status, 0, sizeof(state->status));
}

void mavlink_codec_bench_state_free(struct mavlink_codec_bench_state *state) {
    free(state);
}

/* Decodes `len` bytes one char at a time and returns the number of complete
 * framed messages. Each completed frame is copied into `last_message` so the
 * compiler cannot elide message materialization. */
int mavlink_codec_bench_state_decode(struct mavlink_codec_bench_state *state,
                                     const uint8_t *data, size_t len) {
    int count = 0;
    for (size_t i = 0; i < len; i++) {
        if (mavlink_frame_char_buffer(&state->rxmsg, &state->status, data[i],
                                      &state->r_message,
                                      &state->r_status) == MAVLINK_FRAMING_OK) {
            state->last_message = state->r_message;
            count++;
        }
    }
    return count;
}

const mavlink_message_t *
mavlink_codec_bench_state_last_message(const struct mavlink_codec_bench_state *state) {
    return &state->last_message;
}
