

pub trait Widget {
    /// Natural width of `self`.
    fn width(&self) -> usize;

    /// Draw the widget into a buffer.
    fn draw_into(&self, buffer: &mut dyn std::fmt::Write);

    /// Draw the widget on standard output.
    fn draw(&self) {
        let mut buffer = String::new();
        self.draw_into(&mut buffer);
        println!("{buffer}");
    }
}

pub struct Label {
    label: String,
}

impl Label {
    fn new(label: &str) -> Label {
        Label {
            label: label.to_owned(),
        }
    }
}

pub struct Button {
    label: Label,
    callback: Box<dyn FnMut()>,
}

impl Button {
    fn new(label: &str, callback: Box<dyn FnMut()>) -> Button {
        Button {
            label: Label::new(label),
            callback,
        }
    }
}

pub struct Window {
    title: String,
    widgets: Vec<Box<dyn Widget>>,
}

impl Window {
    fn new(title: &str) -> Window {
        Window {
            title: title.to_owned(),
            widgets: Vec::new(),
        }
    }

    fn add_widget(&mut self, widget: Box<dyn Widget>) {
        self.widgets.push(widget);
    }

    fn inner_width(&self) -> usize {
        std::cmp::max(
            self.title.chars().count(),
            self.widgets.iter().map(|w| w.width()).max().unwrap_or(0),
        )
    }
}


impl Widget for Label {
    fn width(&self) -> usize {
        self.label.chars().count()
    }

    fn draw_into(&self, buffer: &mut dyn std::fmt::Write) {
        match buffer.write_str(self.label.as_str()) {
            Ok(_) => (),
            Err(_) => panic!("Label print went wrong!")
        }
    }
}

impl Widget for Button {
    fn width(&self) -> usize {
        self.label.width()
    }

    fn draw_into(&self, buffer: &mut dyn std::fmt::Write) {
        match buffer.write_str(format!("{}\n", "-".repeat(self.width()).as_str()).as_str()) {
            Ok(_) => (),
            Err(_) => panic!("Button print went wrong")
        }
        self.label.draw_into(buffer);
        match buffer.write_str(format!("\n{}", "-".repeat(self.width()).as_str()).as_str()) {
            Ok(_) => (),
            Err(_) => panic!("Button print went wrong!")
        }
    }
}

impl Widget for Window {
    fn width(&self) -> usize {
        self.inner_width()
    }

    fn draw_into(&self, buffer: &mut dyn std::fmt::Write) {
        match buffer.write_str("*".repeat(self.width()).as_str()) {
            Ok(_) => (),
            Err(_) => panic!("Window print went wrong")
        }
        for w in self.widgets.iter() {
            match buffer.write_str("\n") {
                Ok(_) => (),
                Err(_) => panic!("Window linebreak print went wrong")
            }
            w.draw_into(buffer);
        }
        match buffer.write_str(format!("\n{}", "*".repeat(self.width()).as_str()).as_str()) {
            Ok(_) => (),
            Err(_) => panic!("Window print went wrong")
        }
    }
}

fn main() {
    let mut window = Window::new("Rust GUI Demo 1.23");
    window.add_widget(Box::new(Label::new("This is a small text GUI demo.")));
    window.add_widget(Box::new(Button::new(
        "Click me!",
        Box::new(|| println!("You clicked the button!")),
    )));
    window.draw();
}