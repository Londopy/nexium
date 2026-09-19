/* nexium-gui platform layer: a window, input events, and a framebuffer blit.
 * This is the only C in the GUI. Everything above it (rasterizer, layout,
 * widgets) is Nexium. Imported with @cImport("platform.h"). */
#ifndef NEXIUM_GUI_PLATFORM_H
#define NEXIUM_GUI_PLATFORM_H
#include <stdint.h>

#define GP_EVENT_NONE 0
#define GP_EVENT_CLOSE 1
#define GP_EVENT_MOUSE_MOVE 2
#define GP_EVENT_MOUSE_DOWN 3
#define GP_EVENT_MOUSE_UP 4
#define GP_EVENT_KEY_DOWN 5
#define GP_EVENT_KEY_UP 6
#define GP_EVENT_CHAR 7
#define GP_EVENT_RESIZE 8
#define GP_EVENT_SCROLL 9

/* key codes (a stable subset, independent of the platform's own codes) */
#define GP_KEY_BACKSPACE 8
#define GP_KEY_TAB 9
#define GP_KEY_ENTER 13
#define GP_KEY_ESCAPE 27
#define GP_KEY_SPACE 32
#define GP_KEY_LEFT 256
#define GP_KEY_RIGHT 257
#define GP_KEY_UP 258
#define GP_KEY_DOWN 259
#define GP_KEY_HOME 260
#define GP_KEY_END 261
#define GP_KEY_DELETE 262

typedef struct gp_event {
    int32_t kind;   /* GP_EVENT_* */
    int32_t x;      /* mouse position, or new width for RESIZE, or scroll delta for SCROLL */
    int32_t y;      /* mouse position, or new height for RESIZE */
    int32_t button; /* 1 left, 2 right, 3 middle */
    int32_t key;    /* GP_KEY_* or an ASCII code for KEY_DOWN/KEY_UP */
    int32_t ch;     /* unicode code point for CHAR */
} gp_event;

/* Open the window. Returns 1 on success, 0 when no backend exists on this platform. */
int32_t gp_open(const char* title, int32_t width, int32_t height);
/* Pump the OS queue and pop one event. Returns 1 when `ev` was filled. */
int32_t gp_poll(gp_event* ev);
/* Copy a top-down 0x00RRGGBB framebuffer of the given size to the window. */
void gp_present(const uint32_t* pixels, int32_t width, int32_t height);
int32_t gp_width(void);
int32_t gp_height(void);
void gp_close(void);
/* Seconds since gp_open, with sub-millisecond resolution. */
double gp_time(void);
void gp_sleep(int32_t ms);

#endif
