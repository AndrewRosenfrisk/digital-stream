use core::fmt;
use crossterm::{
    cursor::{Hide, MoveTo},
    execute,
    style::{Attribute, Color, SetAttribute, SetForegroundColor},
    terminal::{
        size, Clear,
        ClearType::{All, Purge},
        DisableLineWrap,
    },
};
use rand::{thread_rng, Rng};
use std::{
    collections::HashMap,
    hash::Hash,
    io::{stdout, Write},
    thread,
    time::Duration,
};

const MIN_STREAM_LENGTH: u8 = 6;
const MAX_STREAM_LENGTH: u8 = 14;
const PAUSE: f32 = 0.1;
const DENSITY: f64 = 0.01;

fn main() -> Result<(), std::io::Error> {
    execute!(stdout(), Hide, Clear(Purge), Clear(All), DisableLineWrap,)?;
    let (columns, rows) = size().unwrap();
    let mut rng = thread_rng();

    let mut screen: HashMap<Point, Pixel> = HashMap::new();
    initialize_screen(columns, rows, &mut screen);
    let mut counter = 0;

    'main: loop {
        let mut new_points: Vec<(Point, u8)> = vec![];
        let mut occupied_columns: Vec<u16> = vec![];
        //update populated pixels
        for pixel in screen.iter_mut() {
            if pixel.1.age > 0 {
                pixel.1.age += 1;
                if pixel.1.age == 2 {
                    //create list of points where we should generate a subsequent pixel
                    occupied_columns.push(pixel.0 .0);
                    new_points.push((Point(pixel.0 .0, pixel.0 .1 + 1), pixel.1.lifetime));
                }
                if pixel.1.age > pixel.1.lifetime {
                    pixel.1.reset();
                } else {
                    occupied_columns.push(pixel.0 .0);
                    pixel.1.set_color();
                    pixel.1.set_attribute();
                }
            }
        }

        //populate next set of pixels
        for point in new_points.into_iter() {
            //find the next pixel down the y axis, if any, and update it to age 1
            let result = screen.get_mut(&Point(point.0 .0, point.0 .1));
            if result.is_some() {
                let next_row = result.unwrap();
                next_row.age = 1;
                next_row.set_color();
                next_row.set_attribute();
                next_row.content = next_row.new_content();
                next_row.lifetime = point.1;
            }
        }
        //choose column for new rain
        for x in 0..=columns {
            if !occupied_columns.contains(&x) && rng.gen_bool(DENSITY) {
                screen.entry(Point(x, 0)).and_modify(|pixel| {
                    //chance to rain
                    *pixel = Pixel {
                        color: Color::Green,
                        attribute: Attribute::NormalIntensity,
                        age: 1,
                        content: pixel.new_content(),
                        lifetime: rng.gen_range(MIN_STREAM_LENGTH..=MAX_STREAM_LENGTH),
                    };
                });
            }
        }

        //print new state across all pixels
        for pixel in screen.iter() {
            print_pixel(pixel.0, pixel.1)?;
        }
        stdout().flush()?;

        counter += 1;
        if counter >= 1000 {
            break 'main;
        }
        thread::sleep(Duration::from_secs_f32(PAUSE));
    }

    println!("Follow the white rabbit...");
    Ok(())
}

#[derive(Hash, Eq, PartialEq)]
struct Point(u16, u16);

#[derive(PartialEq)]
enum Content {
    EMPTY,
    ZERO,
    ONE,
}
impl fmt::Display for Content {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Content::EMPTY => write!(f, " "),
            Content::ZERO => write!(f, "0"),
            Content::ONE => write!(f, "1"),
        }
    }
}

struct Pixel {
    content: Content,
    color: Color,
    attribute: Attribute,
    age: u8,
    lifetime: u8,
}
impl Pixel {
    fn set_color(&mut self) {
        let dark_green_breakpoint = (self.lifetime as f32 * 0.6).floor() as u8;
        let dark_grey_breakpoint = (self.lifetime as f32 * 0.4).floor() as u8;

        if self.age == 1 {
            self.color = Color::Green;
        } else if self.age <= dark_green_breakpoint {
            self.color = Color::DarkGreen;
        } else if self.age <= dark_grey_breakpoint {
            self.color = Color::DarkGrey;
        } else {
            self.color = Color::Black;
        }
    }
    fn set_attribute(&mut self) {
        if self.age > 1 && self.age <= self.lifetime {
            self.attribute = Attribute::Dim;
        } else {
            self.attribute = Attribute::NormalIntensity;
        }
    }
    fn reset(&mut self) {
        *self = Pixel::default();
    }
    fn new_content(&self) -> Content {
        let mut rng = thread_rng();

        if rng.gen_bool(0.5) {
            Content::ONE
        } else {
            Content::ZERO
        }
    }
    fn default() -> Self {
        Pixel {
            content: Content::EMPTY,
            color: Color::Black,
            attribute: Attribute::NormalIntensity,
            age: 0,
            lifetime: 0,
        }
    }
}

fn print_pixel(point: &Point, pixel: &Pixel) -> Result<(), std::io::Error> {
    execute!(
        stdout(),
        MoveTo(point.0, point.1),
        SetForegroundColor(pixel.color),
        SetAttribute(pixel.attribute)
    )?;
    print!("{}", pixel.content);
    Ok(())
}

fn initialize_screen(
    columns: u16,
    rows: u16,
    screen: &mut HashMap<Point, Pixel>,
) -> &mut HashMap<Point, Pixel> {
    for x in 0..columns - 1 {
        for y in 0..rows - 1 {
            screen.insert(Point(x, y), Pixel::default());
        }
    }

    screen
}
