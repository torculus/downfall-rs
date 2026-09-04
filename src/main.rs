use gio::glib;
use gio::prelude::*;
use gtk::gdk;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use keyframe::functions::{EaseInOut, EaseInQuad, Linear};
use rand::RngExt;
use std::cell::RefCell;
use std::rc::Rc;

struct TextCache {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    step: f64,
    duration: f64,
    direction: i32,
    fontfam: &'static str,
}

fn activate(application: &gtk::Application) {
    // Create a normal GTK window however you like
    let window = gtk::ApplicationWindow::new(application);

    // Make sure the window is transparent
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(
        "
            window,
            drawingarea#background {
                background-color: transparent;
                background-image: none;
                box-shadow: none;
                border: none;
            }
            ",
    );

    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );
    }

    // Before the window is first realized, set it up to be a layer surface
    window.init_layer_shell();

    // Display behind normal windows
    window.set_layer(Layer::Bottom);

    // pin the window to all sides
    let anchors = [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom];
    for anchor in anchors {
        window.set_anchor(anchor, true);
    }

    // Set up DrawingArea
    let draw_area = gtk::DrawingArea::new();
    draw_area.set_hexpand(true);
    draw_area.set_vexpand(true);

    let state = Rc::new(RefCell::new(TextCache {
        x0: 0.0,
        x1: 0.0,
        y0: 0.0,
        y1: 0.0,
        step: 0.0,
        duration: 2.0,
        direction: 0,
        fontfam: "Noto Color Emoji",
    }));

    let draw_state = state.clone();
    draw_area.set_draw_func(move |_, cr, _, _| {
        //borrow the state each time
        let draw_state = draw_state.borrow_mut();

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);

        cr.select_font_face(
            draw_state.fontfam,
            gtk::cairo::FontSlant::Normal,
            gtk::cairo::FontWeight::Normal,
        );
        //cr.set_font_size(fontsize);
        cr.set_font_size(26.0);

        let x = keyframe::ease(Linear, draw_state.x0, draw_state.x1, draw_state.step);
        let y = keyframe::ease(EaseInQuad, draw_state.y0, draw_state.y1, draw_state.step);
        cr.move_to(x, y);
        cr.show_text("🍁").unwrap();

        // Blit cached surface directly
        //cr.set_source_surface(&draw_state.text_surface, rounded_x, rounded_y)
        //.unwrap();
        //cr.paint().unwrap();
    });

    let step_state = state.clone();
    draw_area.add_tick_callback(move |widget, _| {
        let mut step_state = step_state.borrow_mut();

        //update the step_state, reset to 0 on overflow
        step_state.step += 0.01 / step_state.duration;
        if step_state.step > 1.0 {
            step_state.step = 0.0;
            (step_state.x0, step_state.x1, step_state.y0, step_state.y1) =
                get_coords(step_state.direction, widget.width(), widget.height());
        }

        //redraw the DrawingArea
        widget.queue_draw();
        glib::ControlFlow::Continue
    });

    window.set_child(Some(&draw_area));
    window.present();
}

fn get_coords(direction: i32, width: i32, height: i32) -> (f64, f64, f64, f64) {
    let mut rng = rand::rng();
    let x0;
    let x1;
    let y0;
    let y1;
    match direction {
        0 => {
            //Down
            x0 = (width as f64) * rng.random_range(0.0..=1.0);
            x1 = (width as f64) * rng.random_range(0.0..=1.0);
            y0 = 0.0;
            y1 = height as f64;
        }
        1 => {
            //Up
            x0 = (width as f64) * rng.random_range(0.0..=1.0);
            x1 = (width as f64) * rng.random_range(0.0..=1.0);
            y0 = height as f64;
            y1 = 0.0;
        }
        2 => {
            //Right
            x0 = 0.0;
            x1 = width as f64;
            y0 = (height as f64) * rng.random_range(0.0..=1.0);
            y1 = (height as f64) * rng.random_range(0.0..=1.0);
        }
        3 => {
            //Left
            x0 = width as f64;
            x1 = 0.0;
            y0 = (height as f64) * rng.random_range(0.0..=1.0);
            y1 = (height as f64) * rng.random_range(0.0..=1.0);
        }
        4 => {
            //Up-Right
            x0 = 0.0;
            x1 = width as f64;
            y0 = (height as f64) * rng.random_range(0.5..=1.5);
            y1 = (height as f64) * rng.random_range(0.0..=0.5);
        }
        5 => {
            //Up-Left
            x0 = width as f64;
            x1 = 0.0;
            y0 = (height as f64) * rng.random_range(0.5..=1.5);
            y1 = (height as f64) * rng.random_range(0.0..=0.5);
        }
        6 => {
            //Down-Right
            x0 = 0.0;
            x1 = width as f64;
            y0 = (height as f64) * rng.random_range(0.0..=0.5);
            y1 = (height as f64) * rng.random_range(0.5..=1.5);
        }
        7 => {
            //Down-Left
            x0 = width as f64;
            x1 = 0.0;
            y0 = (height as f64) * rng.random_range(0.0..=0.5);
            y1 = (height as f64) * rng.random_range(0.5..=1.5);
        }
        _ => {
            //Unpredictable (??)
            x0 = (width as f64) * rng.random_range(0.0..=1.0);
            x1 = (width as f64) * rng.random_range(0.0..=1.0);
            y0 = (height as f64) * rng.random_range(0.0..=1.0);
            y1 = (height as f64) * rng.random_range(0.0..=1.0);
        }
    }
    return (x0, x1, y0, y1);
}

fn main() {
    let application = gtk::Application::new(Some("sh.wmww.gtk-layer-example"), Default::default());

    application.connect_activate(|app| {
        activate(app);
    });

    application.run();
}
