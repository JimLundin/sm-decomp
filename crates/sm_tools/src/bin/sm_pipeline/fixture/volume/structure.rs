trait Shape {
    fn area(&self) -> u32;
}
trait Named {}
struct Square(u32);
impl Shape for Square {
    fn area(&self) -> u32 {
        match self.0 {
            0 => 0,
            side => side * side,
        }
    }
}
impl<T: Shape + ?Sized> Named for T {}
impl<'a, T> Named for &'a Box<T> {}
impl<F: Fn(u8, u8) -> u8> Named for F where F: Copy {}
unsafe impl Send for Square {}
fn make() -> impl Shape {
    Square(1)
}
fn take(shape: impl Shape, call: fn(u8) -> u8) -> u32 {
    shape.area() + u32::from(call(1))
}
macro_rules! twice {
    ($e:expr) => {
        $e + $e
    };
}
