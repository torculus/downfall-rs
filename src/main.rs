use directories::ProjectDirs;
use fastrand;
use gio::glib;
use gio::prelude::*;
use gtk::cairo;
use gtk::gdk;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use keyframe::functions::*;
use pango;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

fn get_config_path() -> Option<PathBuf> {
    // Qualifiers: "com", Organization: "MyOrg", Application: "MyApp"
    let proj_dirs = ProjectDirs::from("com", "torculus", "downfall")?;
    let config_dir = proj_dirs.config_dir();
    fs::create_dir_all(config_dir).ok()?;
    Some(config_dir.join("config.toml"))
}

fn load_config() -> AppSettings {
    let path = match get_config_path() {
        Some(p) => p,
        None => return AppSettings::default(),
    };

    if path.exists() {
        let contents = fs::read_to_string(path).unwrap_or_default();
        toml::from_str(&contents).unwrap_or_else(|_| AppSettings::default())
    } else {
        let config = AppSettings::default();
        save_config(&config);
        config
    }
}

fn save_config(config: &AppSettings) {
    if let Some(path) = get_config_path() {
        if let Ok(toml_string) = toml::to_string_pretty(config) {
            let _ = fs::write(path, toml_string);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub fallstrings: Vec<String>,
    pub max_items: usize,
    pub fontfam: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            fallstrings: vec![String::from("🍁"), String::from("🍂")],
            max_items: 10,
            fontfam: String::from("Noto Color Emoji"),
        }
    }
}

#[derive(Clone)]
struct FallItem {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    step: f64,
    duration: f64,
    direction: i32,
    picture: cairo::ImageSurface,
}

impl FallItem {
    fn change(&self) {}
}

#[derive(Clone)]
struct FIM {
    fallitems: Vec<FallItem>,
    fallpics: Vec<cairo::ImageSurface>,
}

impl FIM {
    fn new(fstr: Vec<String>) -> Self {
        Self {
            fallitems: Vec::new(),
            fallpics: pre_render(fstr),
        }
    }
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

    let app_config = load_config();
    let config_fallstrings = app_config.fallstrings.clone();

    //the state is a FIM
    let state = Rc::new(RefCell::new(FIM::new(config_fallstrings)));

    for _ in 0..=app_config.max_items {
        //snag a random picture out of the (cloned) state.fallpics
        let mut pics = state.borrow_mut().fallpics.clone();
        let index = fastrand::usize(..pics.len());
        let pic = pics.swap_remove(index);

        let item = FallItem {
            x0: fastrand::f64(),
            x1: fastrand::f64(),
            y0: fastrand::f64(),
            y1: fastrand::f64(),
            step: fastrand::f64(),
            duration: 3.0,
            direction: 0,
            picture: pic,
        };
        state.borrow_mut().fallitems.push(item);
    }

    let draw_state = state.clone();
    draw_area.set_draw_func(move |_, cr, _, _| {
        //borrow the state each time
        let draw_state = draw_state.borrow_mut();

        for item in &draw_state.fallitems {
            let x = keyframe::ease(Linear, item.x0, item.x1, item.step);
            let y = keyframe::ease(Linear, item.y0, item.y1, item.step);

            // Blit cached surface directly
            cr.set_source_surface(&item.picture, x, y).unwrap();
            cr.paint().unwrap();
        }
    });

    let step_state = state.clone();
    draw_area.add_tick_callback(move |widget, _| {
        let mut step_state = step_state.borrow_mut();

        //update the step_state, reset to 0 on overflow
        for item in &mut step_state.fallitems {
            //update by 1% of the duration
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

//yes, Google search AI recommended this function
fn pre_render(vecstrings: Vec<String>) -> Vec<cairo::ImageSurface> {
    let mut pics = Vec::new();

    let dummy_surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 1, 1).unwrap();
    let dummy_cr: cairo::Context = cairo::Context::new(dummy_surface).unwrap();
    let pango_ctx = pangocairo::functions::create_context(&dummy_cr);
    let layout = pango::Layout::new(&pango_ctx);

    let mut fontdesc = pango::FontDescription::new();
    fontdesc.set_family("Noto Color Emoji");
    fontdesc.set_weight(pango::Weight::Normal);
    fontdesc.set_absolute_size(26.0 * pango::SCALE as f64);
    layout.set_font_description(Some(&fontdesc));

    for string in vecstrings {
        layout.set_text(string.as_str());

        let (text_width, text_height) = layout.pixel_size();

        // Add extra padding to the surface dimensions so blurred edges aren't clipped
        let padding = 30;
        let surf_w = text_width + (padding * 2);
        let surf_h = text_height + (padding * 2);

        // --- Create Crisp Foreground Layer ---
        let text_surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, surf_w, surf_h).unwrap();
        let cr_text = cairo::Context::new(&text_surface).unwrap();
        cr_text.move_to(padding as f64, padding as f64);
        cr_text.set_source_rgb(1.0, 1.0, 1.0); // White main text
        pangocairo::functions::show_layout(&cr_text, &layout);

        pics.push(text_surface);
    }
    pics
}
fn main() {
    let application = gtk::Application::new(Some("downfall.rs"), Default::default());

    application.connect_activate(|app| {
        activate(app);
    });

    application.run();
}
