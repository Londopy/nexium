/* nexium-gui platform layer. Win32 today; other platforms report "no backend"
 * from gp_open so programs fail cleanly instead of crashing. */
#include "platform.h"
#include <string.h>
#include <stdio.h>

#define GP_QUEUE 512
static gp_event gp_queue[GP_QUEUE];
static int gp_qhead = 0, gp_qtail = 0;
static int gp_w = 0, gp_h = 0;

static void gp_push(gp_event e) {
    int next = (gp_qtail + 1) % GP_QUEUE;
    if (next == gp_qhead) return; /* full: drop */
    gp_queue[gp_qtail] = e;
    gp_qtail = next;
}
static int gp_pop(gp_event* out) {
    if (gp_qhead == gp_qtail) return 0;
    *out = gp_queue[gp_qhead];
    gp_qhead = (gp_qhead + 1) % GP_QUEUE;
    return 1;
}
static gp_event gp_ev(int kind) {
    gp_event e;
    memset(&e, 0, sizeof e);
    e.kind = kind;
    return e;
}

#if defined(_WIN32)
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <windowsx.h>

static HWND gp_hwnd = NULL;
static LARGE_INTEGER gp_freq, gp_t0;

static int gp_map_key(WPARAM vk) {
    switch (vk) {
        case VK_BACK: return GP_KEY_BACKSPACE;
        case VK_TAB: return GP_KEY_TAB;
        case VK_RETURN: return GP_KEY_ENTER;
        case VK_ESCAPE: return GP_KEY_ESCAPE;
        case VK_SPACE: return GP_KEY_SPACE;
        case VK_LEFT: return GP_KEY_LEFT;
        case VK_RIGHT: return GP_KEY_RIGHT;
        case VK_UP: return GP_KEY_UP;
        case VK_DOWN: return GP_KEY_DOWN;
        case VK_HOME: return GP_KEY_HOME;
        case VK_END: return GP_KEY_END;
        case VK_DELETE: return GP_KEY_DELETE;
        default: return (vk >= 0x30 && vk <= 0x5A) ? (int)vk : 0; /* digits and letters as ASCII */
    }
}

static LRESULT CALLBACK gp_wndproc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    gp_event e;
    switch (msg) {
        case WM_CLOSE:
            gp_push(gp_ev(GP_EVENT_CLOSE));
            return 0;
        case WM_DESTROY:
            PostQuitMessage(0);
            return 0;
        case WM_SIZE:
            gp_w = LOWORD(lp); gp_h = HIWORD(lp);
            e = gp_ev(GP_EVENT_RESIZE); e.x = gp_w; e.y = gp_h; gp_push(e);
            return 0;
        case WM_MOUSEMOVE:
            e = gp_ev(GP_EVENT_MOUSE_MOVE); e.x = GET_X_LPARAM(lp); e.y = GET_Y_LPARAM(lp); gp_push(e);
            return 0;
        case WM_LBUTTONDOWN: case WM_RBUTTONDOWN: case WM_MBUTTONDOWN:
            SetCapture(hwnd);
            e = gp_ev(GP_EVENT_MOUSE_DOWN); e.x = GET_X_LPARAM(lp); e.y = GET_Y_LPARAM(lp);
            e.button = msg == WM_LBUTTONDOWN ? 1 : msg == WM_RBUTTONDOWN ? 2 : 3; gp_push(e);
            return 0;
        case WM_LBUTTONUP: case WM_RBUTTONUP: case WM_MBUTTONUP:
            ReleaseCapture();
            e = gp_ev(GP_EVENT_MOUSE_UP); e.x = GET_X_LPARAM(lp); e.y = GET_Y_LPARAM(lp);
            e.button = msg == WM_LBUTTONUP ? 1 : msg == WM_RBUTTONUP ? 2 : 3; gp_push(e);
            return 0;
        case WM_MOUSEWHEEL:
            e = gp_ev(GP_EVENT_SCROLL); e.x = GET_WHEEL_DELTA_WPARAM(wp) / WHEEL_DELTA; gp_push(e);
            return 0;
        case WM_KEYDOWN: case WM_SYSKEYDOWN:
            e = gp_ev(GP_EVENT_KEY_DOWN); e.key = gp_map_key(wp); if (e.key) gp_push(e);
            return msg == WM_KEYDOWN ? 0 : DefWindowProcA(hwnd, msg, wp, lp);
        case WM_KEYUP: case WM_SYSKEYUP:
            e = gp_ev(GP_EVENT_KEY_UP); e.key = gp_map_key(wp); if (e.key) gp_push(e);
            return msg == WM_KEYUP ? 0 : DefWindowProcA(hwnd, msg, wp, lp);
        case WM_CHAR:
            if (wp >= 32 && wp != 127) { e = gp_ev(GP_EVENT_CHAR); e.ch = (int32_t)wp; gp_push(e); }
            return 0;
        case WM_PAINT: {
            PAINTSTRUCT ps;
            BeginPaint(hwnd, &ps);
            EndPaint(hwnd, &ps);
            return 0;
        }
        case WM_ERASEBKGND:
            return 1;
        default:
            return DefWindowProcA(hwnd, msg, wp, lp);
    }
}

int32_t gp_open(const char* title, int32_t width, int32_t height) {
    HINSTANCE inst = GetModuleHandleA(NULL);
    WNDCLASSA wc;
    memset(&wc, 0, sizeof wc);
    wc.lpfnWndProc = gp_wndproc;
    wc.hInstance = inst;
    wc.hCursor = LoadCursor(NULL, IDC_ARROW);
    wc.lpszClassName = "NexiumGuiWindow";
    RegisterClassA(&wc);
    RECT r = { 0, 0, width, height };
    DWORD style = WS_OVERLAPPEDWINDOW;
    AdjustWindowRect(&r, style, FALSE);
    gp_hwnd = CreateWindowExA(0, wc.lpszClassName, title, style, CW_USEDEFAULT, CW_USEDEFAULT, r.right - r.left, r.bottom - r.top, NULL, NULL, inst, NULL);
    if (!gp_hwnd) return 0;
    gp_w = width; gp_h = height;
    QueryPerformanceFrequency(&gp_freq);
    QueryPerformanceCounter(&gp_t0);
    ShowWindow(gp_hwnd, SW_SHOW);
    return 1;
}

int32_t gp_poll(gp_event* ev) {
    MSG msg;
    while (PeekMessageA(&msg, NULL, 0, 0, PM_REMOVE)) {
        TranslateMessage(&msg);
        DispatchMessageA(&msg);
    }
    return gp_pop(ev);
}

void gp_present(const uint32_t* pixels, int32_t width, int32_t height) {
    if (!gp_hwnd) return;
    BITMAPINFO bi;
    memset(&bi, 0, sizeof bi);
    bi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
    bi.bmiHeader.biWidth = width;
    bi.bmiHeader.biHeight = -height; /* top-down */
    bi.bmiHeader.biPlanes = 1;
    bi.bmiHeader.biBitCount = 32;
    bi.bmiHeader.biCompression = BI_RGB;
    HDC dc = GetDC(gp_hwnd);
    StretchDIBits(dc, 0, 0, width, height, 0, 0, width, height, pixels, &bi, DIB_RGB_COLORS, SRCCOPY);
    ReleaseDC(gp_hwnd, dc);
}

int32_t gp_width(void) { return gp_w; }
int32_t gp_height(void) { return gp_h; }

void gp_close(void) {
    if (gp_hwnd) DestroyWindow(gp_hwnd);
    gp_hwnd = NULL;
}

double gp_time(void) {
    LARGE_INTEGER now;
    QueryPerformanceCounter(&now);
    return (double)(now.QuadPart - gp_t0.QuadPart) / (double)gp_freq.QuadPart;
}

void gp_sleep(int32_t ms) { Sleep(ms); }

#else /* no backend yet on this platform */

int32_t gp_open(const char* title, int32_t width, int32_t height) {
    (void)title; (void)width; (void)height;
    fprintf(stderr, "nexium-gui: no window backend for this platform yet (Win32 only)\n");
    return 0;
}
int32_t gp_poll(gp_event* ev) { return gp_pop(ev); }
void gp_present(const uint32_t* pixels, int32_t width, int32_t height) { (void)pixels; (void)width; (void)height; }
int32_t gp_width(void) { return gp_w; }
int32_t gp_height(void) { return gp_h; }
void gp_close(void) {}
double gp_time(void) { return 0.0; }
void gp_sleep(int32_t ms) { (void)ms; }

#endif
