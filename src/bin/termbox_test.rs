
extern crate rustbox;

//use rustbox;

fn main() {
    println!("Hello, rustbox!");
    //let rustbox = match rustbox::RustBox

    let rb = match rustbox::RustBox::init(Default::default()) {
        Ok(v) => v,
        Err(e) => panic!(e),
    };

    rb.print(1, 1, rustbox::RB_BOLD, rustbox::Color::Default, rustbox::Color::Default, "Hello, Bananas!");
    rb.print(1, 3, rustbox::RB_NORMAL, rustbox::Color::Default, rustbox::Color::Default, "Press q to quit.");

    rb.present();

    loop {
        match rb.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    rustbox::Key::Char('q') => { break; }
                    _ => (),
                }
            },
            Err(e) => panic!(e),
            _ => (),
        }
    }
}