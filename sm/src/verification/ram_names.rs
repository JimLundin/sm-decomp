//! The verifier's named work RAM table, from sm_game's RAM name list
//! (`sm_game::ram_table!`): each address with its snesrev name, C type and
//! shape. It names RAM mismatches and spans the oracle's RAM-decoded fields.

/// `Scalar`: the name is one value at `addr`. `Array`: the name is a pointer
/// to the first element of a table of `ty` starting at `addr`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Scalar,
    Array,
}

#[derive(Clone, Copy, Debug)]
pub struct RamVar {
    pub addr: u32,
    pub name: &'static str,
    /// C type name as written in snesrev (`uint16`, `EnemyData`, ...).
    pub ty: &'static str,
    pub shape: Shape,
}

// One row records address, name, original type and scalar/array shape.
// Each group retains ordered names at base + index * stride.
macro_rules! ram_vars {
    ($($base:literal $(stride $stride:literal)?, $ty:ident $shape:ident => [$($name:tt),+ $(,)?];)*) => {
        /// Sorted by address. Several names can share an address; order is significant.
        pub const RAM_VARS: &[RamVar] = &{
            let mut rows = [RamVar { addr: 0, name: "", ty: "", shape: Shape::Scalar };
                0 $(+ [$(ram_vars!(@name $name)),*].len())*];
            let mut index = 0;
            $(
                let first = index;
                let stride = ram_vars!(@stride $($stride)?);
                $(
                    rows[index] = RamVar {
                        addr: $base + (index - first) as u32 * stride,
                        name: ram_vars!(@name $name),
                        ty: stringify!($ty),
                        shape: Shape::$shape,
                    };
                    index += 1;
                )*
            )*
            assert!(index == rows.len());
            rows
        };
    };
    (@stride) => { 0 };
    (@stride $stride:literal) => { $stride };
    (@name $name:literal) => { $name };
    (@name $name:ident) => { stringify!($name) };
}

sm_game::ram_table!(ram_vars);

#[cfg(test)]
mod tests {
    use super::RAM_VARS;
    use sm_game::ram::mirror;

    #[test]
    fn the_register_mirrors_match_the_named_ram_table() {
        for (addr, name) in [
            (mirror::INIDISP, "REG_INIDISP"),
            (mirror::MOSAIC, "REG_MOSAIC"),
            (mirror::BG1SC, "REG_BG1SC"),
            (mirror::BG2SC, "REG_BG2SC"),
            (mirror::TM, "REG_TM"),
            (mirror::COLDATA, "REG_COLDATA"),
            (mirror::SETINI, "REG_SETINI"),
            (mirror::NMITIMEN, "REG_NMITIMEN"),
        ] {
            let row = RAM_VARS.iter().find(|row| row.name == name).expect(name);
            assert_eq!(row.addr as usize, addr, "{name}");
        }
    }
}
