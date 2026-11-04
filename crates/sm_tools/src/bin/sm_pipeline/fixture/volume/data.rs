pub const TABLE: &[(u16, &str)] = &[(1, "a"), (2, "b"), (3, "c")];
pub(crate) static POINTS: [Point; 2] = [Point { x: -1, y: 2 }, Point::ORIGIN];
const ZEROS: [u8; 4] = [0; 4];
static mut COUNTER: Wrapper = Wrapper(0);
const NONE: Option<u8> = None;
const CALL: u16 = u16::from_le_bytes([1, 2]);
const SUM: u8 = 1 + 2;
const BRANCH: u8 = if true { 1 } else { 2 };
const BLOCK: u8 = { 3 };
const CLOSURE: fn(u8) -> u8 = |x| x;
const MACRO: &str = concat!("a", "b");
const fn double(x: u8) -> u8 { x }
struct Array<const N: usize>;
trait Sized2 { const SIZE: usize; }
