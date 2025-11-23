use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct PVTable {
    #[cfg_attr(feature = "serde", serde(with = "serde_handler"))]
    length: Box<[usize; MAX_PLY]>,
    #[cfg_attr(feature = "serde", serde(with = "serde_handler"))]
    table: Box<[SerdeWrapper<[Option<Move>; MAX_PLY]>; MAX_PLY]>,
}

impl PVTable {
    pub fn new() -> Self {
        Self {
            length: Box::new([0; MAX_PLY]),
            table: Box::new([SerdeWrapper::new([None; MAX_PLY]); MAX_PLY]),
        }
    }

    pub fn get_pv(&self, ply: Ply) -> impl Iterator<Item = &Move> {
        get_item_unchecked!(
            self.table,
            ply,
            0..*get_item_unchecked!(@internal self.length, ply)
        )
        .iter()
        .map_while(Option::as_ref)
    }

    pub fn update_table(&mut self, ply: Ply, move_: Move) {
        *get_item_unchecked_mut!(self.table, ply, ply) = Some(move_);
        // let range = (ply + 1)..*get_item_unchecked!(self.length, ply + 1);
        // get_item_unchecked_mut!(self.table, ply, range.clone())
        //     .copy_from_slice(get_item_unchecked!(self.table, ply + 1, range));
        for next_ply in (ply + 1)..*get_item_unchecked!(self.length, ply + 1) {
            *get_item_unchecked_mut!(self.table, ply, next_ply) =
                *get_item_unchecked!(self.table, ply + 1, next_ply);
        }
        self.set_length(ply, *get_item_unchecked!(self.length, ply + 1));
    }

    #[inline]
    pub fn set_length(&mut self, ply: Ply, length: usize) {
        *get_item_unchecked_mut!(self.length, ply) = length;
    }
}

impl Default for PVTable {
    fn default() -> Self {
        Self::new()
    }
}
