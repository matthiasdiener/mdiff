use difference::{Changeset, Difference};
use sourceview::ViewExt;
use std::convert::TryInto;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::Builder;
use std::env::args;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

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
    let scrolled_window1: gtk::ScrolledWindow = builder
        .get_object("scrolled_window1")
        .expect("Couldn't get builder");
    let scrolled_window2: gtk::ScrolledWindow = builder
        .get_object("scrolled_window2")
        .expect("Couldn't get builder");

    let text_view1 = sourceview::View::new();
    let text_view2 = sourceview::View::new();

    scrolled_window1.add(&text_view1);
    scrolled_window2.add(&text_view2);

    text_view1.set_property_monospace(true);
    text_view1.set_draw_spaces(sourceview::DrawSpacesFlags::ALL);
    text_view1.set_wrap_mode(gtk::WrapMode::WordChar);
    text_view2.set_property_monospace(true);
    text_view2.set_wrap_mode(gtk::WrapMode::WordChar);
    text_view2.set_draw_spaces(sourceview::DrawSpacesFlags::ALL);

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
        }
        file_chooser.destroy();
    }));

    text_view1
        .get_buffer()
        .unwrap()
        .connect_changed(clone!(text_view1, text_view2 => move |_| {
            diff(&text_view1.get_buffer().unwrap(), &text_view2.get_buffer().unwrap());
        }));

    text_view2
        .get_buffer()
        .unwrap()
        .connect_changed(clone!(text_view1, text_view2 => move |_| {
            diff(&text_view1.get_buffer().unwrap(), &text_view2.get_buffer().unwrap());
        }));

    let added_tag1 = gtk::TextTag::new(Some("added"));
    added_tag1.set_property_background(Some("green"));

    let removed_tag1 = gtk::TextTag::new(Some("removed"));
    removed_tag1.set_property_background(Some("red"));

    let equal_tag1 = gtk::TextTag::new(Some("equal"));
    equal_tag1.set_property_background(Some("white"));

    let added_tag2 = gtk::TextTag::new(Some("added"));
    added_tag2.set_property_background(Some("green"));

    let removed_tag2 = gtk::TextTag::new(Some("removed"));
    removed_tag2.set_property_background(Some("red"));

    let equal_tag2 = gtk::TextTag::new(Some("equal"));
    equal_tag2.set_property_background(Some("white"));

    text_view1
        .get_buffer()
        .unwrap()
        .get_tag_table()
        .unwrap()
        .add(&added_tag1);
    text_view1
        .get_buffer()
        .unwrap()
        .get_tag_table()
        .unwrap()
        .add(&removed_tag1);
    text_view1
        .get_buffer()
        .unwrap()
        .get_tag_table()
        .unwrap()
        .add(&equal_tag1);

    text_view2
        .get_buffer()
        .unwrap()
        .get_tag_table()
        .unwrap()
        .add(&added_tag2);
    text_view2
        .get_buffer()
        .unwrap()
        .get_tag_table()
        .unwrap()
        .add(&removed_tag2);
    text_view2
        .get_buffer()
        .unwrap()
        .get_tag_table()
        .unwrap()
        .add(&equal_tag2);

    window.show_all();
}

fn diff(b1: &gtk::TextBuffer, b2: &gtk::TextBuffer) {
    b1.remove_all_tags(&b1.get_start_iter(), &b1.get_end_iter());
    b2.remove_all_tags(&b2.get_start_iter(), &b2.get_end_iter());

    let text1 = b1
        .get_slice(&b1.get_start_iter(), &b1.get_end_iter(), true)
        .unwrap();
    let text2 = b2
        .get_slice(&b2.get_start_iter(), &b2.get_end_iter(), true)
        .unwrap();

    let Changeset { diffs, .. } = Changeset::new(&text1, &text2, "");

    let mut iter1 = b1.get_start_iter();

    for c in &diffs {
        match *c {
            Difference::Same(ref z) => {
                let len = z.chars().count();
                let tmpiter = b1.get_iter_at_offset(iter1.get_offset());
                iter1.forward_chars(len.try_into().unwrap());
                b1.apply_tag_by_name("equal", &tmpiter, &iter1);
                // println!("={} ({}-{}, {})", z, tmpiter.get_offset(), iter1.get_offset(), len);
            }
            Difference::Rem(ref z) => {
                let len = z.chars().count();
                let tmpiter = b1.get_iter_at_offset(iter1.get_offset());
                iter1.forward_chars(len.try_into().unwrap());
                b1.apply_tag_by_name("removed", &tmpiter, &iter1);
                // println!("-{} ({}-{}, {})", z, tmpiter.get_offset(), iter1.get_offset(), len);
            }
            _ => (),
        }
    }
    // println!("************************");

    let mut iter2 = b2.get_start_iter();
    for c in &diffs {
        match *c {
            Difference::Same(ref z) => {
                let len = z.chars().count();
                let tmpiter = b2.get_iter_at_offset(iter2.get_offset());
                iter2.forward_chars(len.try_into().unwrap());
                b2.apply_tag_by_name("equal", &tmpiter, &iter2);
                // println!("={} ({}-{}, {})", z, tmpiter.get_offset(), iter2.get_offset(), len);
            }
            Difference::Add(ref z) => {
                let len = z.chars().count();
                let tmpiter = b2.get_iter_at_offset(iter2.get_offset());
                iter2.forward_chars(len.try_into().unwrap());
                b2.apply_tag_by_name("added", &tmpiter, &iter2);
                // println!("+{} ({}-{}, {})", z, tmpiter.get_offset(), iter2.get_offset(), len);
            }
            _ => (),
        }
    }
    // println!("$$$$$$$$$$$$$$$$$$$$$$$");
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
