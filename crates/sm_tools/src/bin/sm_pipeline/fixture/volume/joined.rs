impl<T: Copy> Step for T { fn step(&self, input: u8) -> u8 { match input { 0 => 1, n => n.wrapping_add(1), } } }
const STEPS: [u8; 3] = [1, 2, 3,];
