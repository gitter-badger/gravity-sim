use crate::mouse_state::{MouseResult, MouseState};
use crate::renderer::Renderer;
use crate::simulation::{start_simulation, SimState};

use gdk;
use glib;
use gtk::prelude::*;
use gtk::{self, Application, ApplicationWindow};

use std::cell::RefCell;
use std::rc::Rc;

fn create_drawing_area(
    renderer_rc: &Rc<RefCell<Renderer>>,
    mouse_state: &Rc<RefCell<MouseState>>,
) -> gtk::DrawingArea {
    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_events(
        gdk::EventMask::POINTER_MOTION_MASK
            | gdk::EventMask::POINTER_MOTION_HINT_MASK
            | gdk::EventMask::BUTTON_PRESS_MASK,
    );

    let renderer2 = renderer_rc.clone();
    drawing_area.connect_draw(move |area, cr| {
        let w = area.get_allocated_width() as f64;
        let h = area.get_allocated_height() as f64;
        renderer2.borrow_mut().update_dimensions(w, h);

        renderer2.borrow().render(cr);

        glib::signal::Inhibit(true)
    });

    let renderer3 = renderer_rc.clone();
    let mouse_state1 = mouse_state.clone();
    drawing_area.connect_motion_notify_event(move |_area, event| {
        let (pos_x, pos_y) = event.get_position();
        match mouse_state1
            .borrow_mut()
            .handle_motion(event.get_state(), pos_x, pos_y)
        {
            MouseResult::Drag1(dx, dy) => {
                renderer3.borrow_mut().shift_center(dx, dy);
            }
            MouseResult::Drag2(_dx, dy) => {
                renderer3.borrow_mut().change_zoom(dy);
            }
            _ => (),
        }

        if event.get_is_hint() {
            event.request_motions();
        }

        glib::signal::Inhibit(true)
    });

    drawing_area
}

pub fn build_ui(app: &Application, sim: SimState) {
    let win = ApplicationWindow::new(app);
    let renderer_rc = Rc::new(RefCell::new(Renderer::new(sim.clone(), 0.0, 0.0)));
    let mouse_state = Rc::new(RefCell::new(MouseState::None));

    win.set_title("Gravity simulator");
    win.set_default_size(640, 480);

    let drawing_area = create_drawing_area(&renderer_rc, &mouse_state);

    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    start_simulation(tx, sim);

    let renderer1 = renderer_rc.clone();
    let drawing_area_clone = drawing_area.clone();
    rx.attach(None, move |sim_state| {
        renderer1.borrow_mut().update_state(sim_state);
        drawing_area_clone.queue_draw();

        glib::Continue(true)
    });

    win.add(&drawing_area);

    win.show_all();
}
