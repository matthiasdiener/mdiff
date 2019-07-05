use difference::{Difference, Changeset};
use std::io::Read;
use std::env::args;
use gtk::prelude::*;
use gio::prelude::*;
use std::fs::File;
use gtk::Builder;
use std::io::BufReader;

// use gtk::{Application, ApplicationWindow, TextView};

// upgrade weak reference or return
#[macro_export]
macro_rules! upgrade_weak {
    ($x:ident, $r:expr) => {{
        match $x.upgrade() {
            Some(o) => o,
            None => return $r,
        }
    }};
    ($x:ident) => {
        upgrade_weak!($x, ())
    };
}

macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

pub fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("text_viewer.glade");
    let builder = Builder::new();
    builder
        .add_from_string(glade_src)
        .expect("Couldn't add from string");

    let window: gtk::ApplicationWindow = builder.get_object("window").expect("Couldn't get window");
    window.set_application(Some(application));
    let open_button1: gtk::ToolButton = builder
        .get_object("open_button1")
        .expect("Couldn't get builder");
    let open_button2: gtk::ToolButton = builder
        .get_object("open_button2")
        .expect("Couldn't get builder");    
    let text_view1: gtk::TextView = builder
        .get_object("text_view1")
        .expect("Couldn't get text_view");
    let text_view2: gtk::TextView = builder
        .get_object("text_view2")
        .expect("Couldn't get text_view");

    let window_weak = window.downgrade();
    open_button1.connect_clicked(clone!(text_view1 => move |_| {
        let window = upgrade_weak!(window_weak);

        // TODO move this to a impl?
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Open File"),
            Some(&window),
            gtk::FileChooserAction::Open,
        );
        file_chooser.add_buttons(&[
            ("Open", gtk::ResponseType::Ok),
            ("Cancel", gtk::ResponseType::Cancel),
        ]);
        if file_chooser.run() == gtk::ResponseType::Ok {
            let filename = file_chooser.get_filename().expect("Couldn't get filename");
            let file = File::open(&filename).expect("Couldn't open file");

            let mut reader = BufReader::new(file);
            let mut contents = String::new();
            let _ = reader.read_to_string(&mut contents);

            text_view1
                .get_buffer()
                .expect("Couldn't get window")
                .set_text(&contents);
        }

        file_chooser.destroy();
    }));

    let window_weak = window.downgrade();

    open_button2.connect_clicked(clone!(text_view2 => move |_| {
        let window = upgrade_weak!(window_weak);

        // TODO move this to a impl?
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Open File"),
            Some(&window),
            gtk::FileChooserAction::Open,
        );
        file_chooser.add_buttons(&[
            ("Open", gtk::ResponseType::Ok),
            ("Cancel", gtk::ResponseType::Cancel),
        ]);
        if file_chooser.run() == gtk::ResponseType::Ok {
            let filename = file_chooser.get_filename().expect("Couldn't get filename");
            let file = File::open(&filename).expect("Couldn't open file");

            let mut reader = BufReader::new(file);
            let mut contents = String::new();
            let _ = reader.read_to_string(&mut contents);

            text_view2
                .get_buffer()
                .expect("Couldn't get window")
                .set_text(&contents);
            // text_view2.set_wrap_mode(gtk::WrapMode::Word);
        }
        file_chooser.destroy();
    }));

    text_view1.get_buffer().unwrap().connect_changed(clone!(text_view1 => move |_| {
        // let t1 = text_view1.clone();
        // let t2 = text_view2.clone();
        diff(&text_view1, &text_view2);
    }));

    // text_view2.get_buffer().unwrap().connect_changed(move |_| {
    //     let t1 = text_view1.clone();
    //     let t2 = text_view2.clone();
    //     diff(&t1, &t2);
    // });
    


    window.show_all();
}


fn diff(t1: &gtk::TextView, t2: &gtk::TextView) {
    let b1 = t1.get_buffer().unwrap();
    let b2 = t2.get_buffer().unwrap();
    let text1 = b1.get_text(&b1.get_start_iter(), &b1.get_end_iter(), false).unwrap();
    let text2 = b2.get_text(&b2.get_start_iter(), &b2.get_end_iter(), false).unwrap();

    // let mut t = term::stdout().unwrap();

    // println!("{:?}", text1);

    let Changeset { diffs, .. } = Changeset::new(&text1, &text2, "");

    for c in &diffs {
        match *c {
            Difference::Same(ref z) => {
                // t.fg(term::color::RED).unwrap();
                print!("={}", z);
            }
            Difference::Rem(ref z) => {
                // t.fg(term::color::WHITE).unwrap();
                // t.bg(term::color::RED).unwrap();
                print!("-{}", z);
                // t.reset().unwrap();
            }
            _ => (),
        }
    }
    // t.reset().unwrap();

    // writeln!(t, "");

    for c in &diffs {
        match *c {
            Difference::Same(ref z) => {
                // t.fg(term::color::GREEN).unwrap();
                print!("={}", z);
            }
            Difference::Add(ref z) => {
                // t.fg(term::color::WHITE).unwrap();
                // t.bg(term::color::GREEN).unwrap();
                print!("+{}", z);
                // t.reset().unwrap();
            }
            _ => (),
        }
    }
    println!("");
    // t.reset().unwrap();
    // t.flush().unwrap();
}

fn main() {
    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.text_viewer"),
        Default::default(),
    )
    .expect("Initialization failed...");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
