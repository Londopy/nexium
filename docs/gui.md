# nexium-gui

An immediate-mode GUI written in Nexium. Everything above the window is
Nexium code: a software rasterizer drawing into a `List(u32)` framebuffer, a
bitmap font, layout, and widgets. The only C is `gui/platform.c`, about 200
lines that open a window, queue input events, and blit the framebuffer.

```
import nexium_gui

fn main() {
    var ui = nexium_gui.Ui.open(@cstr("hello"), 400, 200).?
    var clicks: i32 = 0
    var dark = true
    while ui.begin_frame() {
        ui.heading("nexium-gui")
        if ui.button("click me") { clicks += 1 }
        ui.same_line()
        ui.label(format("{} clicks", .{clicks}))
        if ui.checkbox("dark theme", &mut dark) {
            ui.set_theme(if dark { nexium_gui.dark_theme() } else { nexium_gui.light_theme() })
        }
        ui.end_frame()
    }
}
```

Run the demo with `nx run gui/demo.nx`. Render it without a window with
`nx run gui/demo.nx -- --shot frame.ppm`.

## The model

Widgets are function calls made every frame. They draw themselves and return
what happened: `button` returns true on the frame it was clicked, `checkbox`
and `slider` return true when the value changed and write the new value
through the `*mut` you pass. There is no widget tree to build, no callbacks,
and no closures; the application's state is the application's own variables.
This is the egui model, and it suits a language with explicit ownership
because nothing outlives the frame.

A frame is:

1. `begin_frame()` clears the per-frame input flags, pumps the window's
   events, resets the layout cursor and widget ids, and clears the canvas.
   It returns false once the window was closed.
2. Widget calls, top to bottom. Each takes the next id; `hot` (under the
   mouse), `active` (pressed), and `focus` (keyboard) are ids.
3. Free drawing on `ui.canvas` if wanted.
4. `end_frame()` presents the framebuffer and sleeps a few milliseconds.

## API

`Ui`

| call | effect |
| --- | --- |
| `Ui.open(title: *u8, w, h) -> ?Ui` | open a window; `null` when the platform has no backend |
| `Ui.headless(w, h) -> Ui` | a Ui that draws to a canvas without a window (tests, offscreen rendering) |
| `begin_frame() -> bool`, `end_frame()`, `close()` | the frame loop |
| `set_theme(Theme)` | `dark_theme()` and `light_theme()` are provided |
| `label(text)`, `dim_label(text)`, `heading(text)` | text |
| `button(text) -> bool` | true on the frame it is clicked |
| `checkbox(text, *mut bool) -> bool` | true when toggled |
| `slider(text, *mut f32, min, max) -> bool` | true while dragged |
| `text_input(*mut String) -> bool` | click to focus, type, backspace; true when changed |
| `progress(frac)`, `separator()`, `space(px)`, `panel(h)` | layout pieces |
| `same_line()` | the next widget goes to the right of the previous one |
| fields `canvas`, `input`, `theme`, `time`, `dt`, `x`, `y`, `margin` | for free drawing and custom widgets |

`Canvas`

| call | effect |
| --- | --- |
| `Canvas.new(w, h)`, `resize`, `clear(color)` | the framebuffer, `0x00RRGGBB` |
| `set`, `get`, `fill_rect`, `rect`, `fill_rounded`, `hline`, `vline` | clipped primitives |
| `text(x, y, text, color)`, `Canvas.text_width(text)`, `font_h()` | bitmap text, ASCII |
| `save_ppm(path) -> !void` | write the frame as a P6 image |
| `count_color(color)` | pixel count, for tests |

Colors: `rgb(r, g, b)`, `mix(a, b, t)`, `hue_color(h)`.

## Writing a widget

A widget reserves a rectangle, asks `interact` what the mouse did to it, and
draws. This is the whole of `button`:

```
pub fn button(self: *mut Self, text: []u8) -> bool {
    let w = Canvas.text_width(text) + 2 * self.pad
    let h = self.row_h
    let id = self.place(w, h)               // reserve, advance the cursor, take an id
    let x = self.last_x
    let y = self.last_y
    let clicked = self.interact(id, x, y, w, h)
    var fill = self.theme.panel
    if self.hot == id { fill = self.theme.hover }
    if self.active == id { fill = self.theme.active }
    self.canvas.fill_rounded(x, y, w, h, fill)
    self.canvas.rect(x, y, w, h, self.theme.border)
    self.canvas.text(x + self.pad, y + (h - font_h()) / 2, text, self.theme.text)
    return clicked
}
```

## Testing without a window

`Ui.headless` plus direct writes to `ui.input` drive widgets from a `test`
block. Set the input after `begin_frame()`, which clears the per-frame flags:

```
test "button reports a click across frames" {
    var ui = Ui.headless(200, 100)
    expect(ui.begin_frame())
    ui.input.mouse_x = 20
    ui.input.mouse_y = 20
    ui.input.pressed = true
    ui.input.down = true
    expect(!ui.button("ok"))
    ui.end_frame()
    expect(ui.begin_frame())
    ui.input.released = true
    ui.input.down = false
    expect(ui.button("ok"))
    ui.end_frame()
}
```

`nx test gui/nexium_gui.nx` runs these; the test harness runs them too, on every
platform, together with an offscreen render of the demo.

## Platform layer

`gui/platform.h` is eight functions: `gp_open`, `gp_poll`, `gp_present`,
`gp_width`, `gp_height`, `gp_close`, `gp_time`, `gp_sleep`, and one event
struct. `gui/platform.c` implements them with Win32 (GDI `StretchDIBits` for
the blit). On other platforms it compiles to a stub whose `gp_open` returns
0, so the library, the tests, and `--shot` work and only the window is
missing. An X11 or Cocoa backend is the same eight functions.

The library declares its own link line:

```
artifact link { c_sources = ["platform.c"], libs_windows = ["gdi32", "user32"] }
```

## Limits

- ASCII text only, one bitmap size, no anti-aliasing.
- CPU rendering; fine at desktop sizes, not for 4K at 144 Hz.
- Widget ids are call order, so a widget that appears conditionally shifts
  the ids after it for that frame (a press that started on one widget can
  land on another). Keep the widget sequence stable within an interaction.
- No clipping regions, scrolling containers, or windows within the window yet.
