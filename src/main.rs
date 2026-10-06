use fastrand;
use gio::glib;
use gio::prelude::*;
use gtk::gdk;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use keyframe::functions::*;
use std::cell::RefCell;
use std::rc::Rc;

const MAX_ITEMS: usize = 10;

#[derive(Clone)]
struct FallItem {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    step: f64,
    duration: f64,
    direction: i32,
    text: String,
}

struct FIM {
    items: Vec<FallItem>,
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

    let state = Rc::new(RefCell::new(FIM {
        items: Vec::new(),
        fontfam: "Noto Color Emoji",
    }));

    //let falltext = vec![String::from("🍁"), String::from("🙂")];

    for _ in 0..=MAX_ITEMS {
        let item = FallItem {
            x0: 0.0,
            x1: 0.0,
            y0: 0.0,
            y1: 0.0,
            step: fastrand::f64(),
            duration: 3.0,
            direction: 0,
            text: String::from("🍁"),
        };
        state.borrow_mut().items.push(item);
    }

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

        for item in &draw_state.items {
            let x = keyframe::ease(Linear, item.x0, item.x1, item.step);
            let y = keyframe::ease(Linear, item.y0, item.y1, item.step);
            cr.move_to(x, y);
            cr.show_text(item.text.as_str()).unwrap();
        }

        // Blit cached surface directly
        //cr.set_source_surface(&draw_state.text_surface, rounded_x, rounded_y)
        //.unwrap();
        //cr.paint().unwrap();
    });

    let step_state = state.clone();
    draw_area.add_tick_callback(move |widget, _| {
        let mut step_state = step_state.borrow_mut();

        //update the step_state, reset to 0 on overflow
        for item in &mut step_state.items {
            item.step += 0.01 / item.duration;
            if item.step > 1.0 {
                item.step = 0.0;
                (item.x0, item.x1, item.y0, item.y1) =
                    get_coords(item.direction, widget.width(), widget.height());
            }
        }

        //redraw the DrawingArea
        widget.queue_draw();
        glib::ControlFlow::Continue
    });

    window.set_child(Some(&draw_area));
    window.present();
}

fn get_coords(direction: i32, width: i32, height: i32) -> (f64, f64, f64, f64) {
    let x0;
    let x1;
    let y0;
    let y1;
    match direction {
        0 => {
            //Down
            x0 = (width as f64) * fastrand::f64_inclusive();
            x1 = (width as f64) * fastrand::f64_inclusive();
            y0 = 0.0;
            y1 = height as f64;
        }
        1 => {
            //Up
            x0 = (width as f64) * fastrand::f64_inclusive();
            x1 = (width as f64) * fastrand::f64_inclusive();
            y0 = height as f64;
            y1 = 0.0;
        }
        2 => {
            //Right
            x0 = 0.0;
            x1 = width as f64;
            y0 = (height as f64) * fastrand::f64_inclusive();
            y1 = (height as f64) * fastrand::f64_inclusive();
        }
        3 => {
            //Left
            x0 = width as f64;
            x1 = 0.0;
            y0 = (height as f64) * fastrand::f64_inclusive();
            y1 = (height as f64) * fastrand::f64_inclusive();
        }
        4 => {
            //Up-Right
            x0 = 0.0;
            x1 = width as f64;
            y0 = (height as f64) * (fastrand::f64_inclusive() + 0.5);
            y1 = (height as f64) * fastrand::f64_inclusive() * 0.5;
        }
        5 => {
            //Up-Left
            x0 = width as f64;
            x1 = 0.0;
            y0 = (height as f64) * (fastrand::f64_inclusive() + 0.5);
            y1 = (height as f64) * fastrand::f64_inclusive() * 0.5;
        }
        6 => {
            //Down-Right
            x0 = 0.0;
            x1 = width as f64;
            y0 = (height as f64) * fastrand::f64_inclusive() * 0.5;
            y1 = (height as f64) * (fastrand::f64_inclusive() + 0.5);
        }
        7 => {
            //Down-Left
            x0 = width as f64;
            x1 = 0.0;
            y0 = (height as f64) * fastrand::f64_inclusive() * 0.5;
            y1 = (height as f64) * (fastrand::f64_inclusive() + 0.5);
        }
        _ => {
            //Unpredictable (??)
            x0 = (width as f64) * fastrand::f64_inclusive();
            x1 = (width as f64) * fastrand::f64_inclusive();
            y0 = (height as f64) * fastrand::f64_inclusive();
            y1 = (height as f64) * fastrand::f64_inclusive();
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
